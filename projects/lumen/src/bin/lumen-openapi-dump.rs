// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-bin.md#schema
// CODEGEN-BEGIN
//! Print the lumen OpenAPI spec as pretty JSON to stdout.
//!
//! Retained as a back-compat alias; the canonical entrypoint is now
//! `lumen spec [--format openapi]` (see `lumen::spec`). Both delegate to the
//! same offline source — no server boot. Use for codegen: pipe into
//! `openapi-generator` or a `clients/` directory.

fn main() {
    println!("{}", lumen::spec::openapi_json());
}
// CODEGEN-END
