use std::{
    collections::{BTreeSet, HashSet},
    fs::read_dir,
    os::fd::{AsRawFd, OwnedFd},
    path::PathBuf,
};

use log::{error, info};
use nix::sys::socket::{
    AddressFamily, Backlog, SockFlag, SockType, SockaddrIn, bind, listen, socket,
};

use crate::telemetry::force_export_telemetry;

pub fn get_static_file_paths(path: PathBuf) -> HashSet<PathBuf> {
    /*
    Assumption:
    - The parent directory exists and has files in it.

    Consider:
    - Removing recursion here. [x]
    */
    let mut path_bufs: HashSet<PathBuf> = HashSet::new();
    let mut check_directories = BTreeSet::from([path]);

    while !check_directories.is_empty() {
        let directory_path = match check_directories.pop_first() {
            Some(path) => path,
            None => break,
        };
        let dir_reader = match read_dir(directory_path) {
            Ok(reader) => reader,
            Err(e) => {
                error!(error = format!("{}", e).as_str(); "Could Not Read Directory");
                force_export_telemetry(false);
                panic!("Could Not Read Directory | {}", e);
            }
        };

        for result in dir_reader {
            let entry = match result {
                Ok(entry) => entry,
                Err(e) => {
                    error!(error = format!("{}", e).as_str(); "Could Not Read Directory Entry");
                    force_export_telemetry(false);
                    panic!("Could Not Read Directory Entry | {}", e);
                }
            };

            if entry.path().is_dir() {
                check_directories.insert(entry.path());
            } else {
                path_bufs.insert(entry.path());
            }
        }
    }

    return path_bufs;
}

pub fn setup_listening_socket(sock_addr: SockaddrIn, conn_backlog: Backlog) -> OwnedFd {
    let sock_fd = match socket(
        AddressFamily::Inet,
        SockType::Stream,
        SockFlag::empty(),
        None,
    ) {
        Ok(fd) => fd,
        Err(e) => {
            error!(errno = format!("{}", e).as_str(); "Couldn't create listening socket - Cross reference nix & socket(2) docs.");
            force_export_telemetry(false);
            panic!(
                "Couldn't create listening socket - Cross reference nix & socket(2) docs. | {}",
                e
            );
        }
    };

    if let Err(e) = bind(sock_fd.as_raw_fd(), &sock_addr) {
        error!(errno = format!("{}", e).as_str(); "Couldn't bind listening socket to Address - Cross reference nix & bind(2) docs.");
        force_export_telemetry(false);
        panic!(
            "Couldn't bind listening socket to Address - Cross reference nix & bind(2) docs. | {}",
            e
        );
    };

    if let Err(e) = listen(&sock_fd, conn_backlog) {
        error!(errno = format!("{}", e).as_str(); "Couldn't configure listening socket to listen - Cross reference nix & listen(2) docs.");
        force_export_telemetry(false);
        panic!(
            "Couldn't configure listening socket to listen - Cross reference nix & listen(2) docs. | {}",
            e
        );
    };

    info!(listening_address = *sock_addr.to_string().as_str(); "Server listening for requests");
    return sock_fd;
}
