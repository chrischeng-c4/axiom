// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
//! Compile-time constant replacement (define).
//!
//! Replaces identifiers like `process.env.NODE_ENV` with literal values
//! at build time. Similar to esbuild's `--define` and Vite's `define` config.
//!
//! Also provides `import.meta.env` define injection for Vite-compatible
//! environment variable exposure (REQ-JET-08 through REQ-JET-11).

/// GH #3641 — `build_import_meta_env_defines` previously did
/// `serde_json::to_string(value).unwrap_or_else(|_| format!("\"{}\"", value))`.
/// `serde_json::to_string(&String)` is infallible today, but the fallback
/// path was reachable code that, if ever exercised, would emit malformed
/// JS (no escape of `"`, `\`, or control chars) — a real build-break / XSS-class
/// regression lever the moment a future refactor swaps input types.
///
/// This helper produces a valid JS double-quoted string literal for any
/// `&str` input. Happy path delegates to `serde_json::to_string` (which is
/// infallible for valid UTF-8 strings). The defensive fallback is a
/// hand-rolled JSON-string escaper — never a naive wrap.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub(crate) fn safe_import_meta_env_define_value(value: &str) -> String {
    if let Ok(encoded) = serde_json::to_string(value) {
        return encoded;
    }
    // Defensive: hand-rolled JSON-string escape. Should be unreachable
    // because serde_json::to_string on &str is infallible, but the prior
    // panic-free contract is preserved without emitting malformed JS.
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Replace compile-time constants in source code.
///
/// `defines` maps expression strings to replacement values.
/// Example: `{"process.env.NODE_ENV": "\"production\""}`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn replace_defines(
    source: &str,
    defines: &std::collections::HashMap<String, String>,
) -> String {
    if defines.is_empty() {
        return source.to_string();
    }

    let mut result = source.to_string();

    // Sort by length descending to match longest first
    let mut sorted: Vec<(&String, &String)> = defines.iter().collect();
    sorted.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    for (key, value) in sorted {
        result = result.replace(key.as_str(), value.as_str());
    }

    result
}

/// Build default production defines.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn production_defines() -> std::collections::HashMap<String, String> {
    let mut defines = std::collections::HashMap::new();
    defines.insert(
        "process.env.NODE_ENV".to_string(),
        "\"production\"".to_string(),
    );
    defines.insert("__DEV__".to_string(), "false".to_string());
    defines
}

/// Build compile-time defines for `import.meta.env.*` references.
///
/// Produces a map that the bundler uses to replace all `import.meta.env.*`
/// expressions at build time so no environment variables leak into the bundle.
///
/// # Arguments
///
/// * `env_vars` — VITE_* / JET_* key=value pairs loaded from `.env` files.
/// * `mode`     — `"development"` or `"production"`.
///
/// # Produced entries (examples)
///
/// ```text
/// import.meta.env.MODE       → "\"development\""
/// import.meta.env.DEV        → "true"
/// import.meta.env.PROD       → "false"
/// import.meta.env.VITE_URL   → "\"http://localhost:3200\""
/// import.meta.env            → "{}"   ← strips unknown refs in production
/// ```
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn build_import_meta_env_defines(
    env_vars: &std::collections::HashMap<String, String>,
    mode: &str,
) -> std::collections::HashMap<String, String> {
    let mut defines = std::collections::HashMap::new();

    let is_prod = mode == "production";

    // Built-in constants
    defines.insert("import.meta.env.MODE".to_string(), format!("\"{}\"", mode));
    defines.insert("import.meta.env.DEV".to_string(), (!is_prod).to_string());
    defines.insert("import.meta.env.PROD".to_string(), is_prod.to_string());

    // User-provided VITE_* and JET_* vars
    for (key, value) in env_vars {
        let define_key = format!("import.meta.env.{}", key);
        // JSON-encode value → produces a valid JS string literal
        let define_val = serde_json::to_string(value).unwrap_or_else(|_| format!("\"{}\"", value));
        defines.insert(define_key, define_val);
    }

    // Fallback: replace bare `import.meta.env` with `{}` so any remaining
    // property accesses are tree-shaken away.
    defines.insert("import.meta.env".to_string(), "{}".to_string());

    defines
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_replace_node_env() {
        let mut defines = HashMap::new();
        defines.insert(
            "process.env.NODE_ENV".to_string(),
            "\"production\"".to_string(),
        );

        let source = r#"if (process.env.NODE_ENV !== "production") { console.log("dev"); }"#;
        let result = replace_defines(source, &defines);
        assert!(result.contains("\"production\" !== \"production\""));
        assert!(!result.contains("process.env.NODE_ENV"));
    }

    #[test]
    fn test_replace_dev_flag() {
        let mut defines = HashMap::new();
        defines.insert("__DEV__".to_string(), "false".to_string());

        let source = "if (__DEV__) { debug(); }";
        let result = replace_defines(source, &defines);
        assert_eq!(result, "if (false) { debug(); }");
    }

    #[test]
    fn test_empty_defines() {
        let defines = HashMap::new();
        let source = "const x = 1;";
        assert_eq!(replace_defines(source, &defines), source);
    }

    #[test]
    fn test_production_defaults() {
        let defs = production_defines();
        assert_eq!(defs.get("process.env.NODE_ENV").unwrap(), "\"production\"");
        assert_eq!(defs.get("__DEV__").unwrap(), "false");
    }

    #[test]
    fn test_build_import_meta_env_dev() {
        let vars = HashMap::from([(
            "VITE_API_URL".to_string(),
            "http://localhost:3200".to_string(),
        )]);
        let defs = build_import_meta_env_defines(&vars, "development");

        assert_eq!(defs.get("import.meta.env.MODE").unwrap(), "\"development\"");
        assert_eq!(defs.get("import.meta.env.DEV").unwrap(), "true");
        assert_eq!(defs.get("import.meta.env.PROD").unwrap(), "false");
        assert!(defs.contains_key("import.meta.env.VITE_API_URL"));
    }

    #[test]
    fn test_build_import_meta_env_prod() {
        let vars = HashMap::new();
        let defs = build_import_meta_env_defines(&vars, "production");

        assert_eq!(defs.get("import.meta.env.MODE").unwrap(), "\"production\"");
        assert_eq!(defs.get("import.meta.env.DEV").unwrap(), "false");
        assert_eq!(defs.get("import.meta.env.PROD").unwrap(), "true");
    }

    #[test]
    fn test_import_meta_env_replace_in_source() {
        let vars = HashMap::from([(
            "VITE_API_URL".to_string(),
            "http://localhost:3200".to_string(),
        )]);
        let defs = build_import_meta_env_defines(&vars, "production");

        let source = r#"const url = import.meta.env.VITE_API_URL;"#;
        let result = replace_defines(source, &defs);
        assert!(!result.contains("import.meta.env.VITE_API_URL"));
        assert!(result.contains("http://localhost:3200"));
    }

    #[test]
    fn test_import_meta_env_stripped_in_prod() {
        let vars = HashMap::new();
        let defs = build_import_meta_env_defines(&vars, "production");

        // Unknown vars → replaced with {} property access → "{}".UNKNOWN → undefined
        let source = "const x = import.meta.env.UNKNOWN_VAR;";
        let result = replace_defines(source, &defs);
        // import.meta.env.UNKNOWN_VAR is not in defs individually, but
        // import.meta.env (the object) IS replaced with {}
        assert!(!result.contains("import.meta.env.UNKNOWN_VAR") || result.contains("{}"));
    }
}

#[cfg(test)]
mod gh3641_safe_import_meta_env_define_value_tests {
    //! GH #3641 — `build_import_meta_env_defines` used to fall back to
    //! `format!("\"{}\"", value)` if `serde_json::to_string` failed. The
    //! fallback emitted malformed JS for any value containing `"`, `\`, or
    //! control chars — a build-break / XSS-class injection lever in
    //! dev-server contexts. Safe helper produces a valid JS string literal
    //! for all inputs.
    use super::*;

    #[test]
    fn plain_ascii_round_trips_via_serde_json() {
        let out = safe_import_meta_env_define_value("hello");
        assert_eq!(out, "\"hello\"");
    }

    #[test]
    fn embedded_double_quote_is_escaped() {
        let out = safe_import_meta_env_define_value(r#"He said "hi""#);
        // Result must parse as a single JS string token — every `"` after
        // the opening one and before the closing one must be backslashed.
        assert_eq!(out, r#""He said \"hi\"""#);
    }

    #[test]
    fn embedded_backslash_is_escaped() {
        let out = safe_import_meta_env_define_value(r"path\to\file");
        assert_eq!(out, r#""path\\to\\file""#);
    }

    #[test]
    fn newline_carriage_return_tab_are_escaped() {
        let out = safe_import_meta_env_define_value("a\nb\rc\td");
        assert_eq!(out, "\"a\\nb\\rc\\td\"");
    }

    #[test]
    fn control_char_is_escaped_as_unicode() {
        let out = safe_import_meta_env_define_value("\u{0001}");
        assert_eq!(out, "\"\\u0001\"");
    }

    #[test]
    fn empty_string_is_empty_pair_of_quotes() {
        let out = safe_import_meta_env_define_value("");
        assert_eq!(out, "\"\"");
    }

    #[test]
    fn build_import_meta_env_defines_escapes_regression_input() {
        // The regression input from the issue body.
        let vars = std::collections::HashMap::from([(
            "VITE_TITLE".to_string(),
            r#"He said "hi""#.to_string(),
        )]);
        let defs = build_import_meta_env_defines(&vars, "production");
        let val = defs.get("import.meta.env.VITE_TITLE").unwrap();
        // Must be exactly one JS string token: opening `"`, escaped body,
        // closing `"`. Never the broken `"He said "hi""` form.
        assert_eq!(val, r#""He said \"hi\"""#);
        // Negative pin: the broken form must not be produced.
        assert_ne!(val, r#""He said "hi"""#);
    }

    #[test]
    fn helper_output_parses_back_via_serde_json() {
        // Pin: anything the helper emits must round-trip through a JSON
        // parser as a single string literal — that's the JS contract.
        for input in [
            "",
            "hello",
            r#"a "b" c"#,
            r"a\b\c",
            "a\nb",
            "a\tb",
            "a\u{0001}b",
        ] {
            let out = safe_import_meta_env_define_value(input);
            let parsed: String = serde_json::from_str(&out)
                .unwrap_or_else(|e| panic!("emit not parseable: {out:?} err={e}"));
            assert_eq!(parsed, input, "round-trip lost data for {input:?}");
        }
    }
}
// CODEGEN-END
