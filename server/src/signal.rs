use std::process::exit;

use log::error;
use nix::{
    libc::c_int,
    sys::signal::{SaFlags, SigAction, SigHandler, SigSet, Signal, sigaction},
};

use crate::statics::SHUTDOWN_SERVER;
use crate::telemetry::force_export_telemetry;

// NOTE Start:
// not sure how to handle logging with signals
// don't know if telemetry export function is signal safe
extern "C" fn sig_handler(_signal: c_int) {
    match SHUTDOWN_SERVER.write() {
        Ok(mut guard) => {
            if *guard {
                force_export_telemetry(false); // don't know if this signal safe
                exit(0);
            }
            *guard = true;
        }
        Err(poisoned_guard) => {
            *poisoned_guard.into_inner() = true;
        }
    }
}

pub fn setup_sig_handler() {
    let sig_act = SigAction::new(
        SigHandler::Handler(sig_handler),
        SaFlags::empty(),
        SigSet::empty(),
    );

    if let Err(e) = unsafe { sigaction(Signal::SIGINT, &sig_act) } {
        error!(errno = format!("{}", e).as_str(); "Could not set up signal handler");
        force_export_telemetry(false);
        panic!("Could not set up signal handler | {}", e);
    };
}
// NOTE End:
