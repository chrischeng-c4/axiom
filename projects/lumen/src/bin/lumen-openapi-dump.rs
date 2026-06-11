// <HANDWRITE gap="standardize:claim-code" tracker="projects-lumen-src-bin-lumen-openapi-dump-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
//! Print the lumen OpenAPI spec as pretty JSON to stdout.
//!
//! Retained as a back-compat alias; the canonical entrypoint is now
//! `lumen spec [--format openapi]` (see `lumen::spec`). Both delegate to the
//! same offline source — no server boot. Use for codegen: pipe into
//! `openapi-generator` or a `clients/` directory.

fn main() {
    println!("{}", lumen::spec::openapi_json());
}

// </HANDWRITE>
