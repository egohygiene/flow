//! Thin initial product delivery. Native backend/cross-platform qualification is deferred.
use flow::command::delivery::{self, Presentation};
use std::{
    io::Write,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
#[path = "support/help_branding.rs"]
mod help_branding;
fn cancellation() -> Result<Arc<AtomicBool>, flow::command::Error> {
    let token = Arc::new(AtomicBool::new(false));
    #[cfg(unix)]
    {
        use nix::sys::signal::{SigSet, Signal};
        let mut signals = SigSet::empty();
        signals.add(Signal::SIGINT);
        signals.add(Signal::SIGTERM);
        signals.thread_block().map_err(|_| flow::command::Error {
            code: flow::command::Code::Io,
            message: "cannot configure CLI cancellation".into(),
            recovery_directory: None,
        })?;
        let shared = token.clone();
        std::thread::spawn(move || {
            let _ = signals.wait();
            shared.store(true, Ordering::Relaxed);
        });
    }
    Ok(token)
}
fn main() -> std::process::ExitCode {
    let values = std::env::args_os().skip(1).collect::<Vec<_>>();
    let root_help = help_branding::literal_root_help(&values);
    // A malformed machine request still receives JSON if it selected JSON.
    let fallback = if values
        .windows(2)
        .any(|w| w[0] == "--format" && w[1] == "json")
    {
        Presentation::Json
    } else {
        Presentation::Human
    };
    let (result, presentation) = match delivery::parse(values) {
        Err(error) => (delivery::failure("parse", error), fallback),
        Ok(args) => {
            let presentation = args.presentation;
            let result = match cancellation() {
                Err(e) => delivery::failure(&args.command, e),
                Ok(token) => delivery::execute(&args, &|| token.load(Ordering::Relaxed)),
            };
            (result, presentation)
        }
    };
    let (bytes, code) = match delivery::render(&result, presentation) {
        Ok(bytes) => (bytes, result.exit_code),
        Err(error) => {
            let failed = delivery::failure("presentation", error);
            match delivery::render(&failed, presentation) {
                Ok(bytes) => (bytes, failed.exit_code),
                Err(_) => return std::process::ExitCode::from(8),
            }
        }
    };
    // No provider work remains. Restore ordinary interruption while a possibly
    // redirected stdout blocks; the library never owns process signal masks.
    #[cfg(unix)]
    {
        use nix::sys::signal::{SigSet, Signal};
        let mut signals = SigSet::empty();
        signals.add(Signal::SIGINT);
        signals.add(Signal::SIGTERM);
        let _ = signals.thread_unblock();
    }
    if root_help && code == 0 && result.command == "help" && presentation == Presentation::Human {
        if std::io::stdout()
            .lock()
            .write_all(b"flow - compose independent local tools\n")
            .is_err()
        {
            return std::process::ExitCode::from(8);
        }
        help_branding::write_if_selected(&[std::ffi::OsString::from("--help")]);
    }
    if std::io::stdout().lock().write_all(&bytes).is_err() {
        return std::process::ExitCode::from(8);
    }
    std::process::ExitCode::from(code)
}
