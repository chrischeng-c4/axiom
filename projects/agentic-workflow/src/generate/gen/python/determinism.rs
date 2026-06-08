// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-generate-gen-python.md#schema
// CODEGEN-BEGIN
// Output normalization for the Python emitter (R4 of
// projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md). Hand-written until the determinism
// primitives library (LF newlines, trailing-whitespace strip, single
// trailing newline) is shared across emitter backends.

/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#py-normalize
pub fn normalize(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 1);
    for line in input.replace("\r\n", "\n").split('\n') {
        out.push_str(line.trim_end());
        out.push('\n');
    }
    while out.ends_with("\n\n") {
        out.pop();
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}
// CODEGEN-END
