// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
// CODEGEN-BEGIN
//! `jet-parity-gate` — binary entrypoint. Delegates to `cli::run` and
//! maps the returned exit code onto `std::process::exit`.

fn main() {
    let code = jet_parity_gate::cli::run();
    std::process::exit(code);
}
// CODEGEN-END
