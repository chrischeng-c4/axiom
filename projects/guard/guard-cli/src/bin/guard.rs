// SPEC-MANAGED: projects/guard/tech-design/semantic/source/projects-guard-guard-cli-src-bin-guard-rs.md#rust-source-unit
// CODEGEN-BEGIN
use std::process::ExitCode;

use clap::Parser;
use guard_cli::{dispatch, print_report, GuardCommand};

fn main() -> ExitCode {
    let cmd = GuardCommand::parse();
    let out = cmd.output.clone();
    let report = dispatch(cmd);
    print_report(&report, &out);
    ExitCode::from(report.exit_code.clamp(0, 255) as u8)
}
// CODEGEN-END
