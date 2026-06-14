// SPEC-MANAGED: projects/cap/tech-design/semantic/cap-src.md#schema
// CODEGEN-BEGIN
use std::process::ExitCode;

use cap::cli;

fn main() -> ExitCode {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("cap: failed to start runtime: {e}");
            return ExitCode::FAILURE;
        }
    };
    match rt.block_on(cli::run()) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("cap: {e:#}");
            ExitCode::FAILURE
        }
    }
}
// CODEGEN-END
