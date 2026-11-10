//! `fs-resources`: one binary for everything (ADR 0063). Pattern: Facade over the crates that stay
//! libraries.
//!
//! ```text
//! fs-resources                  the window (feature `gui`, on by default)
//! fs-resources [--data DIR] [--size WxH] [--snapshot F.png]
//! fs-resources serve            the server (Cap'n Proto, resources.capnp: types, reports)
//! ```

use std::process::ExitCode;

use fs_resources_server::ServeError;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("serve") => serve(&args[1..]),
        Some("gui") => window(&args[1..]),
        _ => window(&args),
    }
}

fn serve(words: &[String]) -> ExitCode {
    match fs_resources_server::serve(words.iter().map(Into::into)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(ServeError::Usage(m)) => {
            eprintln!("fs-resources: {m}");
            ExitCode::from(2)
        }
        Err(ServeError::Failed(m)) => {
            eprintln!("fs-resources: {m}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(feature = "gui")]
fn window(words: &[String]) -> ExitCode {
    match fs_resources_app::gui::run(words) {
        Ok(()) => ExitCode::SUCCESS,
        Err(m) => {
            eprintln!("fs-resources: {m}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(feature = "gui"))]
fn window(_words: &[String]) -> ExitCode {
    eprintln!(
        "fs-resources: this build has no window (built with --no-default-features); use `fs-resources serve`"
    );
    ExitCode::FAILURE
}
