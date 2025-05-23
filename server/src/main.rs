mod init;
mod serve;
mod signal;
mod statics;
mod telemetry;
use nix::sys::socket::SockaddrIn;
use serve::Server;
use signal::setup_sig_handler;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    path::PathBuf,
    time::Duration,
};
use telemetry::{init_telemetry, shutdown_telemetry};

fn main() {
    let (log_provider, metrics_provider, tracer_provider) = init_telemetry();
    setup_sig_handler();
    /*
    TODO Start:
    test?? - only file gathering and integration test
    - use an atomic for the shutdown signal
    - dockerize the application.
    - next time have an abstraction over the server router
    - export to a optel collector
    - put the path of static files in the logs
    - reorg init module into serve module
    - remove hardcoding from routering function
    - ignore the syscall interrupted signals when flag is set
    */
    let mut server = {
        let sock_addr = SockaddrIn::from(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080));
        let static_files_location = PathBuf::from("../client/dist");
        let timeout = Duration::from_millis(400);
        Server::init_server(timeout, static_files_location, sock_addr)
    };
    server.begin_connection_handlers();
    server.accept_connections_and_send_to_handlers();
    server.wait_for_handlers_to_finish();

    shutdown_telemetry(log_provider, metrics_provider, tracer_provider);
}
