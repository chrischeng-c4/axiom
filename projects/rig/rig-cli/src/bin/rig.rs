//! `rig` standalone binary: parse → dispatch → one JSON document → exit code.

use clap::Parser;
use rig_cli::dispatch::{execute, print_report, RigCommand};

fn main() {
    let cmd = RigCommand::parse();
    let output = cmd.output.clone();
    let report = execute(cmd);
    let code = print_report(&report, &output);
    std::process::exit(code);
}
