//! trybuild harness for `#[derive(AgentSchema)]` compile-time errors.
//!
//! Each fixture under `tests/compile_fail/*.rs` must fail to compile
//! with a clear `agent-derive` error message. trybuild snapshots the
//! diagnostic in a matching `.stderr` file (auto-generated on first
//! run via `TRYBUILD=overwrite cargo test`).
//
// HANDWRITE-BEGIN reason: trybuild harness scaffolding is not yet a
// known section type for codegen.

#[test]
fn compile_fail_diagnostics() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/tuple_field.rs");
    t.compile_fail("tests/compile_fail/reference_field.rs");
}

// HANDWRITE-END
