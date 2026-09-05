//! `cfgtool validate <file>` prints `ok: version <n>, workers <auto or count>` and exits 0, or
//! prints the reason on stderr and exits 2. `cfgtool migrate <in> <out>` writes the version 2
//! form of `<in>` to `<out>` with the same exit codes, and writes nothing when `<in>` does not load.

use std::env;
use std::path::Path;
use std::process::ExitCode;

/// The exit code for a file that does not validate, or a bad command line.
const FAILURE: u8 = 2;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["validate", path] => cfgtool::load(Path::new(path)).map_err(|error| error.to_string()),
        ["migrate", from, to] => {
            cfgtool::migrate(Path::new(from), Path::new(to)).map_err(|error| error.to_string())
        }
        _ => Err("usage: cfgtool validate <file> | cfgtool migrate <in> <out>".to_owned()),
    };
    match result {
        Ok(config) => {
            println!("ok: version {}, workers {}", config.version, config.workers);
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("cfgtool: {message}");
            ExitCode::from(FAILURE)
        }
    }
}
