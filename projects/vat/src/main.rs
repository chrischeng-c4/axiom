// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-src.md#schema
// CODEGEN-BEGIN
use std::process::ExitCode;

fn main() -> ExitCode {
    match vat::cli::run() {
        Ok(code) => code,
        Err(err) => {
            // Print the full anyhow chain so an agent reading stderr gets the
            // root cause, not just the top-level message.
            eprintln!("vat: error: {err:#}");
            ExitCode::FAILURE
        }
    }
}
// CODEGEN-END
