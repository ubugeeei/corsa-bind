use std::{env, path::PathBuf, process::ExitCode};

use corsa_bind_core::fast::{CompactString, SmallVec, compact_format};
use corsa_bind_ref::CorsaRefManager;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> corsa_bind_core::Result<()> {
    let args = env::args()
        .skip(1)
        .map(CompactString::from)
        .collect::<SmallVec<[CompactString; 4]>>();
    let command = args.first().map(CompactString::as_str).unwrap_or("verify");
    let lock_path = args
        .get(1)
        .map(|path| PathBuf::from(path.as_str()))
        .unwrap_or_else(|| PathBuf::from("corsa_origin.lock.toml"));
    let manager = CorsaRefManager::new(lock_path);
    match command {
        "status" => {
            let status = manager.status()?;
            println!("{}", status.describe());
            Ok(())
        }
        "verify" => manager.verify(),
        "sync" => manager.sync(),
        "pin-current" => manager.pin_current(),
        other => Err(corsa_bind_core::CorsaError::Protocol(compact_format(
            format_args!("unknown corsa_origin command: {other}"),
        ))),
    }
}
