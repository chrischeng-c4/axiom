---
id: sdd-generate-gen-rust-tests
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# TestsGenOutput Type

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/gen/rust/tests.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `TestsGenOutput` | projects/agentic-workflow/src/generate/gen/rust/tests.rs | struct | pub | 74 |  |
| `generate_e2e_tests` | projects/agentic-workflow/src/generate/gen/rust/tests.rs | function | pub | 169 | generate_e2e_tests(spec_content: &str) -> TestsGenOutput |
| `generate_tests` | projects/agentic-workflow/src/generate/gen/rust/tests.rs | function | pub | 86 | generate_tests(spec_content: &str) -> TestsGenOutput |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  TestsGenOutput:
    type: object
    required: [code, emitted, test_count]
    description: |
      Output from test cases generation.
    properties:
      code:
        type: string
        description: "The rendered test file body (no CODEGEN markers — apply.rs wraps them)."
      emitted:
        type: boolean
        description: "Whether content was produced."
      test_count:
        type: integer
        x-rust-type: "usize"
        description: "Number of test fns emitted, exposed for diagnostics."
    x-rust-struct:
      derive: [Debug, Clone]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/gen/rust/tests.rs -->
~~~rust
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/gen/rust/tests.md#source
// CODEGEN-BEGIN
//! Test cases generator — emits runnable `#[test]` bodies from a `tests:`
//! YAML list in the spec's `## Tests` section.
//!
//! Rust generator template contract:
//!
//! ```yaml
//! preamble: |
//!   //! Optional module docs, helper functions, and other raw Rust emitted
//!   //! before imports/tests.
//! postamble: |
//!   fn trailing_helper() {}
//! imports:
//!   - "use mambalibs_http::http_exception::HTTPException;"
//!   - "use std::collections::HashMap;"
//! module:
//!   attributes: ["#[cfg(test)]"]
//!   name: http_exception_tests
//!
//! tests:
//!   - name: preserves_explicit_detail
//!     leading: |
//!       // REQ: REQ-001
//!     attributes: ["#[test]"]
//!     setup: |
//!       let exc = HTTPException::new(
//!           404, Some("custom".into()), HashMap::new(),
//!       ).unwrap();
//!     assertions:
//!       - 'assert_eq!(exc.status_code, 404)'
//!       - 'assert_eq!(exc.detail, "custom")'
//!
//!   - name: validation_rejects_out_of_range
//!     setup: |
//!       let result = HTTPException::new(600, None, HashMap::new());
//!     assertions:
//!       - "assert!(result.is_err())"
//!
//!   - name: main
//!     test: false
//!     attributes: []
//!     body: |
//!       println!("smoke");
//! ```
//!
//! Output is a Rust file body: optional raw `file_preamble`, optional module
//! wrapper, optional raw in-module `preamble`, the import block, one test
//! function per entry, and optional `postamble`. These fields are template data
//! for the Rust generator, not new global section types. `attributes` defaults
//! to `#[test]`; `async: true` renders `async fn`; `body` may provide the whole
//! raw function body. `indent_body: false` preserves raw body indentation for
//! cases such as Rust raw string fixtures where adding spaces would change the
//! test data. `test: false` suppresses the default `#[test]` attribute for
//! smoke harnesses and examples. Without `body`, the `setup` block is emitted
//! verbatim and each assertion is terminated with a `;`.
//!
//! Pragmatic compromise: we embed raw Rust expressions in the YAML rather
//! than inventing an assertion DSL. The resulting `.rs` file is still 100%
//! codegen-produced from the spec — just with the spec-author writing the
//! assertion expressions directly.

use serde::Serialize;
use serde_yaml::Value;

use crate::generate::engine::TemplateEngine;

const TPL_TEST_FILE: &str = include_str!("templates/tests/test_file.tera");

/// Output from test cases generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/tests.md#schema
#[derive(Debug, Clone)]
pub struct TestsGenOutput {
    /// The rendered test file body (no CODEGEN markers — apply.rs wraps them).
    pub code: String,
    /// Whether content was produced.
    pub emitted: bool,
    /// Number of test fns emitted, exposed for diagnostics.
    pub test_count: usize,
}

/// Render the `## Tests` section of a spec into a Rust test file body.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/tests.md#source
pub fn generate_tests(spec_content: &str) -> TestsGenOutput {
    let Some(yaml_text) = crate::generate::apply::extract_section_yaml(spec_content, "Tests")
    else {
        return TestsGenOutput {
            code: String::new(),
            emitted: false,
            test_count: 0,
        };
    };
    let yaml: Value = match serde_yaml::from_str(&yaml_text) {
        Ok(v) => v,
        Err(_) => {
            return TestsGenOutput {
                code: String::new(),
                emitted: false,
                test_count: 0,
            }
        }
    };

    let module = parse_module(&yaml);
    let body_indent_spaces = if module.enabled { 8 } else { 4 };
    let imports = parse_imports(&yaml);
    let member_indent_spaces = if module.enabled { 4 } else { 0 };
    let tests = parse_tests(&yaml, body_indent_spaces, member_indent_spaces);
    if tests.is_empty() {
        return TestsGenOutput {
            code: String::new(),
            emitted: false,
            test_count: 0,
        };
    }

    let mut engine = TemplateEngine::empty();
    engine
        .add_template("test_file.tera", TPL_TEST_FILE)
        .expect("test_file.tera parse");

    let ctx = TestsContext {
        file_preamble: parse_file_preamble(&yaml),
        preamble: parse_preamble(&yaml, module.enabled),
        postamble: parse_postamble(&yaml, module.enabled),
        imports,
        tests: tests.clone(),
        has_module: module.enabled,
        module_attributes: module.attributes,
        module_name: module.name,
        member_indent: if module.enabled {
            "    ".to_string()
        } else {
            String::new()
        },
    };
    let code = engine
        .render("test_file.tera", &ctx)
        .expect("test_file.tera render")
        .trim_end()
        .to_string();

    TestsGenOutput {
        code,
        emitted: true,
        test_count: tests.len(),
    }
}

// ── context ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct TestsContext {
    file_preamble: String,
    preamble: String,
    postamble: String,
    imports: Vec<String>,
    tests: Vec<TestCase>,
    has_module: bool,
    module_attributes: Vec<String>,
    module_name: String,
    member_indent: String,
}

#[derive(Debug, Clone, Serialize)]
struct TestCase {
    name: String,
    /// Optional comments emitted immediately before the test attributes.
    leading: String,
    attributes: Vec<String>,
    visibility: String,
    async_prefix: String,
    return_type: String,
    /// Multi-line raw Rust function body. Emitted verbatim.
    body: String,
}

#[derive(Debug, Default)]
struct ModuleConfig {
    enabled: bool,
    attributes: Vec<String>,
    name: String,
}

fn parse_file_preamble(yaml: &Value) -> String {
    yaml.get("file_preamble")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim_end()
        .to_string()
}

fn parse_preamble(yaml: &Value, in_module: bool) -> String {
    let preamble = yaml
        .get("preamble")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim_end();
    if preamble.is_empty() {
        String::new()
    } else if in_module {
        indent_rust_block_with(preamble, 4)
    } else {
        preamble.to_string()
    }
}

fn parse_postamble(yaml: &Value, in_module: bool) -> String {
    let postamble = yaml
        .get("postamble")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim_end();
    if postamble.is_empty() {
        String::new()
    } else if in_module {
        indent_rust_block_with(postamble, 4)
    } else {
        postamble.to_string()
    }
}

fn parse_module(yaml: &Value) -> ModuleConfig {
    let Some(module) = yaml.get("module").and_then(|v| v.as_mapping()) else {
        return ModuleConfig::default();
    };
    let Some(name) = module.get("name").and_then(|v| v.as_str()) else {
        return ModuleConfig::default();
    };
    let name = name.trim();
    if name.is_empty() {
        return ModuleConfig::default();
    }
    let attributes = module
        .get("attributes")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    ModuleConfig {
        enabled: true,
        attributes,
        name: name.to_string(),
    }
}

fn parse_imports(yaml: &Value) -> Vec<String> {
    yaml.get("imports")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_tests(
    yaml: &Value,
    body_indent_spaces: usize,
    member_indent_spaces: usize,
) -> Vec<TestCase> {
    let Some(seq) = yaml.get("tests").and_then(|v| v.as_sequence()) else {
        return Vec::new();
    };
    seq.iter()
        .filter_map(|entry| {
            let m = entry.as_mapping()?;
            let name = m.get("name").and_then(|v| v.as_str())?.to_string();
            let leading = m
                .get("leading")
                .and_then(|v| v.as_str())
                .map(str::trim_end)
                .filter(|s| !s.is_empty())
                .map(|s| indent_rust_block_with(s, member_indent_spaces))
                .unwrap_or_default();
            let attributes = if let Some(seq) = m.get("attributes").and_then(|v| v.as_sequence()) {
                seq.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            } else if m.get("test").and_then(|v| v.as_bool()).unwrap_or(true) {
                vec!["#[test]".to_string()]
            } else {
                Vec::new()
            };
            let visibility = m
                .get("visibility")
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
                .map(|s| format!("{} ", s.trim()))
                .unwrap_or_default();
            let async_prefix = if m.get("async").and_then(|v| v.as_bool()).unwrap_or(false) {
                "async ".to_string()
            } else {
                String::new()
            };
            let return_type = m
                .get("return_type")
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
                .map(|s| format!(" -> {}", s.trim()))
                .unwrap_or_default();
            let assertions: Vec<String> = m
                .get("assertions")
                .and_then(|v| v.as_sequence())
                .map(|seq| {
                    seq.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let indent_body = m
                .get("indent_body")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let body = if let Some(body) = m.get("body").and_then(|v| v.as_str()) {
                let body = body.trim_end();
                if indent_body {
                    indent_rust_block_with(body, body_indent_spaces)
                } else {
                    body.to_string()
                }
            } else {
                let setup = m
                    .get("setup")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim_end()
                    .to_string();
                let mut body = setup;
                for assertion in assertions {
                    if !body.is_empty() {
                        body.push('\n');
                    }
                    body.push_str(&assertion);
                    body.push(';');
                }
                indent_rust_block_with(&body, body_indent_spaces)
            };
            Some(TestCase {
                name,
                leading,
                attributes,
                visibility,
                async_prefix,
                return_type,
                body,
            })
        })
        .collect()
}

fn indent_rust_block_with(block: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    block
        .lines()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("{indent}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_tests_section_emits_nothing() {
        let spec = "## Overview\nNothing here.\n";
        let out = generate_tests(spec);
        assert!(!out.emitted);
        assert_eq!(out.test_count, 0);
    }

    fn tests_spec(yaml: &str) -> String {
        let mut out = String::from("\n## Tests\n<!-- type: tests lang: yaml -->\n\n```yaml\n");
        out.push_str(yaml);
        if !yaml.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("```\n");
        out
    }

    #[test]
    fn empty_tests_list_emits_nothing() {
        let spec = tests_spec("imports: []\ntests: []\n");
        let out = generate_tests(&spec);
        assert!(!out.emitted);
    }

    #[test]
    fn renders_single_test_with_imports_and_assertions() {
        let spec = tests_spec(
            r###"imports:
  - "use mambalibs_http::http_exception::HTTPException;"
  - "use std::collections::HashMap;"
tests:
  - name: preserves_explicit_detail
    setup: |
      let exc = HTTPException::new(
          404, Some("custom".into()), HashMap::new(),
      ).unwrap();
    assertions:
      - 'assert_eq!(exc.status_code, 404)'
      - 'assert_eq!(exc.detail, "custom")'
"###,
        );
        let out = generate_tests(&spec);
        assert!(out.emitted);
        assert_eq!(out.test_count, 1);
        assert!(out.code.contains("use mambalibs_http::http_exception::HTTPException;"));
        assert!(out.code.contains("use std::collections::HashMap;"));
        assert!(out.code.contains("#[test]"));
        assert!(out.code.contains("fn preserves_explicit_detail()"));
        assert!(out.code.contains("let exc = HTTPException::new"));
        assert!(out.code.contains("assert_eq!(exc.status_code, 404);"));
        assert!(out.code.contains("assert_eq!(exc.detail, \"custom\");"));
    }

    #[test]
    fn renders_multiple_tests() {
        let spec = tests_spec(
            r#"imports: ["use mambalibs_http::http_exception::HTTPException;"]
tests:
  - name: ok_case
    setup: "let result = HTTPException::new(404, None, Default::default());"
    assertions:
      - "assert!(result.is_ok())"
  - name: err_case
    setup: "let result = HTTPException::new(600, None, Default::default());"
    assertions:
      - "assert!(result.is_err())"
"#,
        );
        let out = generate_tests(&spec);
        assert!(out.emitted);
        assert_eq!(out.test_count, 2);
        assert!(out.code.contains("fn ok_case()"));
        assert!(out.code.contains("fn err_case()"));
        assert!(out.code.contains("assert!(result.is_ok());"));
        assert!(out.code.contains("assert!(result.is_err());"));
    }

    #[test]
    fn renders_preamble_attributes_async_and_raw_body() {
        let spec = tests_spec(
            r##"preamble: |
  //! module docs

  fn helper() -> bool { true }
imports:
  - "use std::time::Duration;"
tests:
  - name: async_ignored_case
    attributes:
      - "#[tokio::test]"
      - "#[ignore = \"needs external service\"]"
    async: true
    body: |
      assert!(helper());
      let _timeout = Duration::from_secs(1);
"##,
        );
        let out = generate_tests(&spec);
        assert!(out.emitted);
        assert_eq!(out.test_count, 1);
        assert!(out.code.contains("//! module docs"));
        assert!(out.code.contains("fn helper() -> bool { true }"));
        assert!(out.code.contains("use std::time::Duration;"));
        assert!(out.code.contains("#[tokio::test]"));
        assert!(out.code.contains("#[ignore = \"needs external service\"]"));
        assert!(out.code.contains("async fn async_ignored_case()"));
        assert!(out.code.contains("    assert!(helper());"));
    }

    #[test]
    fn can_preserve_raw_body_indentation() {
        let spec = tests_spec(
            r###"imports: []
tests:
  - name: raw_string_fixture
    indent_body: false
    body: |
      let fixture = r#"---
      id: raw
      ## Heading
      "#;
      assert!(fixture.contains("## Heading"));
"###,
        );
        let out = generate_tests(&spec);
        assert!(out.emitted);
        assert!(out.code.contains("let fixture = r#\"---"));
        assert!(out.code.contains("\nid: raw\n## Heading\n"));
        assert!(out
            .code
            .contains("\nassert!(fixture.contains(\"## Heading\"));"));
    }

    #[test]
    fn can_render_smoke_harness_without_test_attribute() {
        let spec = tests_spec(
            r###"file_preamble: |
  //! Smoke harness.
tests:
  - name: main
    test: false
    attributes: []
    return_type: std::process::ExitCode
    body: |
      std::process::ExitCode::SUCCESS
"###,
        );
        let out = generate_tests(&spec);
        assert!(out.emitted);
        assert_eq!(out.test_count, 1);
        assert!(out.code.contains("//! Smoke harness."));
        assert!(!out.code.contains("#[test]"));
        assert!(out.code.contains("fn main() -> std::process::ExitCode {"));
        assert!(out.code.contains("    std::process::ExitCode::SUCCESS"));
    }

    #[test]
    fn renders_module_wrapper_with_helpers() {
        let spec = tests_spec(
            r###"file_preamble: |
  //! File docs.
module:
  attributes:
    - "#[cfg(test)]"
  name: generated_tests
preamble: |
  use super::*;

  fn helper() -> bool { true }
postamble: |
  fn trailing_helper() -> bool { true }
imports:
  - "use std::path::PathBuf;"
tests:
  - name: helper_is_available
    leading: |
      // REQ: REQ-001
    body: |
      let _path = PathBuf::from("fixture");
      assert!(helper());
"###,
        );
        let out = generate_tests(&spec);
        assert!(out.emitted);
        assert!(out.code.contains("//! File docs."));
        assert!(out.code.contains("#[cfg(test)]\nmod generated_tests {"));
        assert!(out.code.contains("    use super::*;"));
        assert!(out.code.contains("    fn helper() -> bool { true }"));
        assert!(out.code.contains("    use std::path::PathBuf;"));
        assert!(out.code.contains("    // REQ: REQ-001"));
        assert!(out.code.contains("    #[test]"));
        assert!(out.code.contains("    fn helper_is_available() {"));
        assert!(out
            .code
            .contains("        let _path = PathBuf::from(\"fixture\");"));
        assert!(out
            .code
            .contains("    fn trailing_helper() -> bool { true }"));
        assert!(out.code.ends_with("\n}"));
    }
}

// CODEGEN-END
~~~

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/gen/rust/tests.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete test cases generator module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Pure data carrier; matches gen output struct precedent.
- [schema] Single struct; bool + usize bare in `required:`.
- [changes] Standard split.
