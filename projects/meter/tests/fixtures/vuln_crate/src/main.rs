// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-tests-fixtures-vuln-crate-src-main-rs.md#source
// CODEGEN-BEGIN
//! Intentionally vulnerable fixture crate for the meter audit trust-bug test.
//! It pins `time = "=0.1.45"` (RUSTSEC-2020-0071) so `cargo audit` reports
//! a vulnerability. It is never built as part of the real workspace.
fn main() {}
// CODEGEN-END
