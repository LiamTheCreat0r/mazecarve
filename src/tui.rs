//! The live view (placeholder until the ratatui loop lands).

use std::process::ExitCode;

use crate::options::Options;

pub fn run(_o: &mut Options) -> ExitCode {
    eprintln!("mazecarve: live mode is not implemented yet, use -p");
    ExitCode::from(1)
}
