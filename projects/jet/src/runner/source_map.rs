// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
// CODEGEN-BEGIN
//! Source map generation for JIT-transformed files.
//!
//! Generates inline source maps so that Node.js stack traces
//! point back to the original .ts/.tsx source lines.

use base64::Engine;

/// Append an inline source map comment to JavaScript code.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub fn append_inline_source_map(code: &str, source_map_json: &str) -> String {
    let encoded = base64::engine::general_purpose::STANDARD.encode(source_map_json.as_bytes());
    format!(
        "{}\n//# sourceMappingURL=data:application/json;base64,{}\n",
        code, encoded
    )
}

/// Generate a minimal V3 source map JSON for a 1:1 line mapping.
///
/// This is used when type stripping doesn't change line numbers,
/// so each output line maps to the same input line.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub fn generate_identity_source_map(source_file: &str, source_content: &str) -> String {
    let line_count = source_content.lines().count();
    // Each line: "AAAA" = (0,0,0,0) relative to previous
    let mappings = (0..line_count)
        .map(|_| "AAAA")
        .collect::<Vec<_>>()
        .join(";");

    serde_json::json!({
        "version": 3,
        "file": source_file.replace(".ts", ".js")
            .replace(".tsx", ".js")
            .replace(".jsx", ".js"),
        "sources": [source_file],
        "sourcesContent": [source_content],
        "names": [],
        "mappings": mappings,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_source_map() {
        let code = "console.log(42);";
        let sm = r#"{"version":3}"#;
        let result = append_inline_source_map(code, sm);
        assert!(result.starts_with("console.log(42);"));
        assert!(result.contains("sourceMappingURL=data:application/json;base64,"));
    }

    #[test]
    fn test_identity_source_map() {
        let sm = generate_identity_source_map("test.ts", "line1\nline2\nline3");
        let parsed: serde_json::Value = serde_json::from_str(&sm).unwrap();
        assert_eq!(parsed["version"], 3);
        assert_eq!(parsed["sources"][0], "test.ts");
        let mappings = parsed["mappings"].as_str().unwrap();
        assert_eq!(mappings.matches(';').count(), 2); // 3 lines = 2 semicolons
    }
}
// CODEGEN-END
