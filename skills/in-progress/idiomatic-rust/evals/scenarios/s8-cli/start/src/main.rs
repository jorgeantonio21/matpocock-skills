//! `cfgtool validate <file>`: prints `ok: version <n>, workers <count>` and exits 0, or prints the
//! reason on stderr and exits 2.

use std::env;
use std::path::Path;
use std::process::ExitCode;

/// The exit code for a file that does not validate, or a bad command line.
const FAILURE: u8 = 2;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let [command, path] = args.as_slice() else {
        eprintln!("usage: cfgtool validate <file>");
        return ExitCode::from(FAILURE);
    };
    if command != "validate" {
        eprintln!("cfgtool: unknown command {command:?}; the one command is validate");
        return ExitCode::from(FAILURE);
    }
    match cfgtool::load(Path::new(path)) {
        Ok(config) => {
            println!("ok: version {}, workers {}", config.version, config.workers);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("cfgtool: {error}");
            ExitCode::from(FAILURE)
        }
    }
}
