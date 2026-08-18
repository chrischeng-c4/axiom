// CODEGEN-BEGIN
use std::process::ExitCode;

fn main() -> ExitCode {
    let docker_shim = vat::docker_shim::invoked_as_docker();
    let result = if docker_shim {
        vat::docker_shim::run_from_env()
    } else {
        vat::cli::run()
    };
    match result {
        Ok(code) => code,
        Err(err) => {
            // Print the full anyhow chain so an agent reading stderr gets the
            // root cause, not just the top-level message.
            let program = if docker_shim { "docker" } else { "vat" };
            eprintln!("{program}: error: {err:#}");
            ExitCode::FAILURE
        }
    }
}
// CODEGEN-END
