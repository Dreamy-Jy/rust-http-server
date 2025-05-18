use std::{
    collections::HashSet,
    fs,
    os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd},
    path::PathBuf,
    thread::{JoinHandle, available_parallelism},
    time::Duration,
};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, unbounded};
use log::{error, info, warn};
use nix::{
    errno::Errno,
    poll::{PollFd, PollFlags, PollTimeout, poll},
    sys::socket::{Backlog, MsgFlags, SockaddrIn, accept, getpeername, recv, send},
};
use opentelemetry::{
    KeyValue, global,
    metrics::Counter,
    trace::{Span, SpanKind, Tracer},
};
use std::str;

use crate::{
    init::{get_static_file_paths, setup_listening_socket},
    statics::SHUTDOWN_SERVER,
    telemetry::{force_export_telemetry, get_tracer},
};

enum SysCallError {
    Timeout,
    Error(MessagedErrno),
}

struct MessagedErrno {
    errno: Errno,
    message: String,
}

#[derive(Clone)]
struct ConnectionChannel {
    sender: Option<Sender<OwnedFd>>,
    receiver: Receiver<OwnedFd>,
}

pub struct Server {
    static_files: HashSet<PathBuf>,
    total_reqs: Counter<u64>,
    finished_reqs: Counter<u64>,
    listening_sock: OwnedFd,
    timeout: Duration,
    cxns: ConnectionChannel,
    join_handlers: Option<Vec<JoinHandle<()>>>,
}

impl Server {
    pub fn init_server(
        timeout: Duration,
        static_files_location: PathBuf,
        sock_addr: SockaddrIn,
    ) -> Self {
        let static_files = get_static_file_paths(static_files_location);
        if static_files.is_empty() {
            error!("No static files found");
            force_export_telemetry(false);
            panic!("No static files found")
        }

        let reqs_started = global::meter("requests")
            .u64_counter("total_started")
            .with_description("Total number of requests started")
            .build();
        let reqs_finished = global::meter("requests")
            .u64_counter("total_finished")
            .with_description("Total number of requests finished")
            .build();

        let listening_sock = setup_listening_socket(sock_addr, Backlog::MAXCONN);

        let conns_chanel = {
            let (sender, receiver) = unbounded();
            ConnectionChannel {
                sender: Some(sender),
                receiver,
            }
        };

        return Server {
            static_files,
            total_reqs: reqs_started,
            finished_reqs: reqs_finished,
            listening_sock,
            timeout,
            cxns: conns_chanel,
            join_handlers: None,
        };
    }

    pub fn begin_connection_handlers(&mut self) {
        let thread_count = match available_parallelism() {
            Ok(threads) => threads.get(),
            Err(e) => {
                warn!(
                    error = format!("{}", e).as_str();
                    "Rust available_parallelism failed - only a single request thread will be spawned"
                );
                1
            }
        };

        let mut join_handlers: Vec<JoinHandle<_>> = Vec::new();

        for thread_id in 0..thread_count {
            let total_reqs = self.total_reqs.clone();
            let finished_reqs = self.finished_reqs.clone();
            let static_files = self.static_files.clone();
            let timeout = self.timeout.clone();
            let receiver = self.cxns.receiver.clone();

            let join_handler = std::thread::spawn(move || {
                loop {
                    if let Ok(flag) = SHUTDOWN_SERVER.read() {
                        if *flag {
                            break;
                        }
                    }

                    let conn_fd = match receiver.recv_timeout(timeout) {
                        Ok(fd) => fd,
                        Err(RecvTimeoutError::Timeout) => continue,
                        Err(RecvTimeoutError::Disconnected) => break,
                    };

                    handle_request(
                        &total_reqs,
                        &finished_reqs,
                        thread_id,
                        conn_fd,
                        &static_files,
                        &timeout,
                    );
                }
            });

            join_handlers.push(join_handler);
        }

        self.join_handlers = Some(join_handlers);
    }

    pub fn accept_connections_and_send_to_handlers(&mut self) {
        assert!(
            self.cxns.sender.is_some(),
            "connection channel sender must be initialized"
        );
        let sender = match self.cxns.sender.take() {
            Some(sender) => sender,
            None => {
                // this is poorly handled
                error!("Connection channel sender not initialized");
                force_export_telemetry(false);
                panic!("Connection channel sender not initialized");
            }
        };

        loop {
            if let Ok(flag) = SHUTDOWN_SERVER.read() {
                if *flag {
                    break;
                }
            }

            let conn_fd = {
                let mut poll_targets =
                    [PollFd::new(self.listening_sock.as_fd(), PollFlags::POLLIN)];
                let timeout = match PollTimeout::try_from(self.timeout) {
                    Ok(timeout) => timeout,
                    Err(e) => {
                        warn!(error = format!("{}", e).as_str(); "Defaulting to non-blocking timeout - couldn't set polling timeout");
                        PollTimeout::ZERO
                    }
                };

                if let Err(e) = poll(&mut poll_targets, timeout) {
                    error!(error = format!("{}", e).as_str(); "Skipping request - poll failed");
                    continue;
                }
                match poll_targets[0].revents() {
                    // continue if connection is available
                    Some(PollFlags::POLLIN) => (),
                    // skip if poll timedout
                    Some(_) | None => continue,
                }

                match accept(self.listening_sock.as_raw_fd()) {
                    Ok(fd) => unsafe { OwnedFd::from_raw_fd(fd) },
                    Err(e) => {
                        error!(error = format!("{}", e).as_str(); "Skipping request - accept failed");
                        continue;
                    }
                }
            };

            if let Err(e) = sender.send(conn_fd) {
                // might cause issues with blocking
                error!(error = format!("{}", e).as_str(); "Skipping request - could not send connection to handlers");
                continue;
            }
        }
    }

    pub fn wait_for_handlers_to_finish(&mut self) {
        match self.join_handlers.take() {
            Some(handlers) => {
                for handler in handlers {
                    if let Err(_) = handler.join() {
                        error!("Thread Join Failed");
                    }
                }
            }
            None => {
                error!("No handlers to join");
                force_export_telemetry(false);
                panic!("No handlers to join");
            }
        }
    }
}

fn read_request(fd: BorrowedFd, poll_timeout_duration: Duration) -> Result<Vec<u8>, SysCallError> {
    /*
    Assumption:
    - Client allways sends a request, before server sends a response.
    ! What basis do we have for this? (Check HTTP protocol specification)
    */
    let mut read_buf = [0u8; 1000];
    let mut req: Vec<u8> = Vec::new();

    let timeout = match PollTimeout::try_from(poll_timeout_duration) {
        Ok(timeout) => timeout,
        Err(e) => {
            warn!(error = format!("{}", e).as_str(); "Defaulting to polling timeout max - couldn't set polling timeout");
            PollTimeout::MAX
        }
    };
    let mut poll_targets = [PollFd::new(fd, PollFlags::POLLIN)];

    if let Err(e) = poll(&mut poll_targets, timeout) {
        return Err(SysCallError::Error(MessagedErrno {
            errno: e,
            message: "Skipping request - poll failed".to_string(),
        }));
    };
    match poll_targets[0].revents() {
        Some(PollFlags::POLLIN) => (),
        Some(_) | None => return Err(SysCallError::Timeout),
    }
    loop {
        match recv(fd.as_raw_fd(), &mut read_buf[..], MsgFlags::MSG_DONTWAIT) {
            Ok(req_size) => {
                req.extend_from_slice(&read_buf[..req_size]);
                if req_size < read_buf.len() {
                    break;
                }
            }
            Err(e) => {
                if e == Errno::EAGAIN || e == Errno::EWOULDBLOCK {
                    break;
                } else {
                    return Err(SysCallError::Error(MessagedErrno {
                        errno: e,
                        message: "Skipping request - recv had unexpected error".to_string(),
                    }));
                }
            }
        }
    }

    Ok(req)
}

fn build_response(req: Vec<u8>, static_files: &HashSet<PathBuf>) -> String {
    /*
    Assumption:
    - We'll always get a request.
    - The request is a valid HTTP request.
    */
    let status_line_parts = {
        // This variable is the status line of an HTTP request split by spaces.
        let end_of_line_position = match req.iter().position(|&b| b == b'\r') {
            Some(pos) => pos,
            None => {
                error!("Invalid HTTP Request - Could Not Find End Of Line");
                let message = "Bad Request";
                return format!(
                    "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                    message.len(),
                    message
                );
            }
        };
        let status_line = String::from_utf8_lossy(&req[..end_of_line_position]);
        status_line
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    };
    let requested_path = {
        let path_string = String::from("../client/dist") + status_line_parts[1].as_str();
        PathBuf::from(path_string)
    };

    match (status_line_parts[0].as_str(), status_line_parts[1].as_str()) {
        ("GET", _path) if static_files.contains(&requested_path) => {
            let content_type = {
                let file_ext = requested_path.extension();

                match file_ext.and_then(|ex| ex.to_str()) {
                    Some("html") => "text/html",
                    Some("css") => "text/css",
                    Some("js") => "application/javascript",
                    Some("svg") => "image/svg+xml",
                    None => "application/json",
                    _ => "application/octet-stream",
                }
            };
            let content = match fs::read_to_string(requested_path.display().to_string()) {
                Ok(content) => content,
                Err(e) => {
                    error!(error = format!("{}", e).as_str(); "Could Not Read File");
                    let message = "Internal Server Error | Could Not Read File";
                    return format!(
                        "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\n\r\n{}",
                        message.len(),
                        message
                    );
                }
            };

            return format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                content_type,
                content.len(),
                content
            );
        }
        ("GET", "/") => {
            let content = match fs::read_to_string("../client/dist/index.html") {
                Ok(content) => content,
                Err(e) => {
                    error!(error = format!("{}", e).as_str(); "Could Not Read File");
                    let message = "Internal Server Error | Could Not Read File";
                    return format!(
                        "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\n\r\n{}",
                        message.len(),
                        message
                    );
                }
            };
            return format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\n\r\n{}",
                content.len(),
                content
            );
        }
        (_method, _path) => {
            let message = "Resource Not Found";
            return format!(
                "HTTP/1.1 404 Not Found\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\n\r\n{}",
                message.len(),
                message
            );
        }
    }
}

pub fn handle_request(
    total_counter: &Counter<u64>,
    success_counter: &Counter<u64>,
    thread_id: usize,
    conn_fd: OwnedFd,
    serve_files: &HashSet<PathBuf>,
    poll_timeout: &Duration,
) {
    total_counter.add(1, &[]);
    let tracer = get_tracer();
    let mut span = tracer
        .span_builder("request")
        .with_kind(SpanKind::Server)
        .start(tracer);

    span.set_attribute(KeyValue::new("thread_id", thread_id as i64));
    let mut is_warning = false;

    let caller_addr = match getpeername::<SockaddrIn>(conn_fd.as_raw_fd()) {
        Ok(sock_addr) => {
            span.set_attribute(KeyValue::new("caller_address", sock_addr.to_string()));
            sock_addr.to_string()
        }
        Err(_) => {
            is_warning = true;
            "don't know".to_string()
        }
    };

    let req: Vec<u8> = match read_request(conn_fd.as_fd(), *poll_timeout) {
        Ok(req) => req,
        Err(SysCallError::Timeout) => return,
        Err(SysCallError::Error(e)) => {
            error!(thread_id = thread_id, errno = format!("{}", e.errno).as_str(); "{}", e.message);
            return;
        }
    };
    let request_string = match str::from_utf8(req.as_slice()) {
        Ok(s) => s,
        Err(_) => {
            is_warning = true;
            "Couldn't convert"
        }
    };

    let resp = build_response(req.clone(), serve_files);
    if let Err(e) = send(conn_fd.as_raw_fd(), resp.as_bytes(), MsgFlags::empty()) {
        error!(thread_id = thread_id, errno = format!("{}", e).as_str(); "Skipping request - could not send data to socket");
        return;
    };

    success_counter.add(1, &[]);
    if is_warning {
        warn!(
            thread_id = thread_id,
            caller_address = caller_addr.as_str(),
            request = request_string;
            /*response = *resp.as_str();*/
            "Request handled with warnings"
        );
    } else {
        info!(
            thread_id = thread_id,
            caller_address = caller_addr.as_str(),
            request = request_string;
            /*response = *resp.as_str();*/
            "Request successfully handled"
        );
    }
    return;
}
