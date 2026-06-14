// SPEC-MANAGED: projects/arena/tech-design/semantic/source/projects-arena-arena-cli-src-bin-arena-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `arena` standalone binary: parse → dispatch → one JSON document → exit code.

use arena_cli::dispatch::{execute, print_report, ArenaCommand};
use clap::Parser;

fn main() {
    let cmd = ArenaCommand::parse();
    let output = cmd.output.clone();
    let report = execute(cmd);
    let code = print_report(&report, &output);
    std::process::exit(code);
}
// CODEGEN-END
