// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-main-rs.md#rust-source-unit
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
// SPEC-MANAGED: projects/vat/tech-design/logic/vat-td-ast-promote-remaining-grouped-source-units.md#rust-source-unit
// CODEGEN-BEGIN

// CODEGEN-END
