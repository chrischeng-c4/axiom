---
id: sdd-generate-generators-primitive-registry
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Primitive Registry Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/generators/primitive_registry.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `PrimitiveEntry` | projects/agentic-workflow/src/generate/generators/primitive_registry.rs | struct | pub | 27 |  |
| `REGISTRY` | projects/agentic-workflow/src/generate/generators/primitive_registry.rs | constant | pub | 58 |  |
| `is_prose_section` | projects/agentic-workflow/src/generate/generators/primitive_registry.rs | function | pub | 269 | is_prose_section(t: SectionType) -> bool |
| `kind_to_name` | projects/agentic-workflow/src/generate/generators/primitive_registry.rs | function | pub | 282 | kind_to_name(kind: &PrimitiveKind) -> &'static str |
| `lookup` | projects/agentic-workflow/src/generate/generators/primitive_registry.rs | function | pub | 195 | lookup(kind: &PrimitiveKind) -> Option<&'static PrimitiveEntry> |
| `lookup_by_name` | projects/agentic-workflow/src/generate/generators/primitive_registry.rs | function | pub | 207 | lookup_by_name(name: &str) -> Option<&'static PrimitiveEntry> |
| `substitute_template` | projects/agentic-workflow/src/generate/generators/primitive_registry.rs | function | pub | 218 | substitute_template(template: &str, bindings: &[(&str, &str)]) -> String |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/generators/primitive_registry.rs -->
```rust

//! Primitive vocabulary registry for Mermaid Plus flowchart code generation.
//!
//! Provides a static lookup table mapping [`PrimitiveKind`] to [`PrimitiveEntry`]
//! values that the [`super::logic_primitive_emitter::LogicPrimitiveEmitter`] uses
//! to emit Rust code from flowchart YAML nodes annotated with `primitive:` fields.
//!
//! ## Regenerability boundary
//!
//! The full module is source-template managed. The primitive vocabulary spec
//! remains the semantic contract for the registry entries, while this source
//! template owns the concrete Rust table and helper functions until schema-to-
//! registry-table emission exists.
//!
//! @spec projects/agentic-workflow/tech-design/surface/specs/mermaid-plus-primitive-vocabulary.md#schema

use crate::generate::diagrams::flowchart_plus::PrimitiveKind;
use crate::models::spec_rules::SectionType;

/// Full specification for one primitive, stored in the static REGISTRY.
///
/// Fields map directly to the `PrimitiveEntry` YAML schema definition in the spec.
/// @spec projects/agentic-workflow/tech-design/surface/specs/mermaid-plus-primitive-vocabulary.md#schema
#[derive(Debug, Clone)]
pub struct PrimitiveEntry {
    /// The primitive discriminant (`primitive:` field value in YAML).
    pub name: &'static str,
    /// Abstract Rust function signature template for documentation purposes.
    /// Uses `{var}` placeholders matching the `inputs` field names.
    pub signature: &'static str,
    /// Rust emit template with `{var}` placeholders for input bindings
    /// and `{out}` for the output binding variable name.
    pub emit_template: &'static str,
    /// Whether the operation is fallible — when true, `?` is appended.
    pub fallible: bool,
    /// Abstract output type token (e.g. `"string"`, `"bool"`, `"unit"`, `"T"`).
    pub output_type: &'static str,
}

/// Static primitive vocabulary registry: 15 entries.
///
/// 12 MVP + 3 added by score-chat-jsonl-migration self-bootstrap:
///   - parse_jsonl_stream  (closes gap-blocker #parse-jsonl-stream)
///   - append_line_atomic  (closes gap-blocker #append-line-atomic)
///   - run_subprocess      (partial closure of #tail-f-stream — emits the
///     spawn step; RAII Drop guard + stream loop remain deferred because
///     primitives emit single statements, not multi-line patterns. A
///     follow-up SDD issue is needed for block-level codegen.)
///
/// Remaining deferred: walk_up_to_marker, parse_toml, parse_markdown,
/// serialize_json, serialize_toml, truncate_at, join_with_separator, for_each,
/// reduce, find, filter, sort_by, match, sleep, format_timestamp, print_stderr,
/// read_stdin, terminal.
///
/// @spec projects/agentic-workflow/tech-design/surface/specs/mermaid-plus-primitive-vocabulary.md#schema
pub const REGISTRY: &[PrimitiveEntry] = &[
    // ── File IO ─────────────────────────────────────────────────────────────
    PrimitiveEntry {
        name: "read_file",
        signature: "fn read_file(path: &str) -> Result<String>",
        emit_template: "let {out} = std::fs::read_to_string({path})?;",
        fallible: true,
        output_type: "string",
    },
    PrimitiveEntry {
        name: "write_file",
        signature: "fn write_file(path: &str, content: &str) -> Result<()>",
        emit_template: "std::fs::write({path}, {content})?;",
        fallible: true,
        output_type: "unit",
    },
    PrimitiveEntry {
        name: "append_file",
        signature: "fn append_file(path: &str, content: &str) -> Result<()>",
        emit_template: "{ use std::io::Write; let mut f = std::fs::OpenOptions::new().append(true).open({path})?; write!(f, \"{{}}\", {content})?; }",
        fallible: true,
        output_type: "unit",
    },
    PrimitiveEntry {
        name: "path_exists",
        signature: "fn path_exists(path: &str) -> bool",
        emit_template: "let {out} = std::path::Path::new({path}).exists();",
        fallible: false,
        output_type: "bool",
    },
    // ── JSONL stream IO (closes gap-blockers from score-chat-jsonl-migration) ─
    PrimitiveEntry {
        name: "parse_jsonl_stream",
        signature: "fn parse_jsonl_stream<T: DeserializeOwned>(path: &str) -> Vec<T>",
        emit_template: "let {out}: Vec<{T}> = std::fs::File::open({path}).map(|f| { use std::io::BufRead; std::io::BufReader::new(f).lines().map_while(Result::ok).filter(|l| !l.trim().is_empty()).filter_map(|l| serde_json::from_str(&l).ok()).collect() }).unwrap_or_default();",
        fallible: false,
        output_type: "Vec<T>",
    },
    PrimitiveEntry {
        name: "append_line_atomic",
        signature: "fn append_line_atomic<T: Serialize>(path: &str, value: &T) -> Result<()>",
        emit_template: "{ use std::io::Write; let line = format!(\"{{}}\\n\", serde_json::to_string(&{value})?); let mut f = std::fs::OpenOptions::new().create(true).append(true).open({path})?; f.write_all(line.as_bytes())?; }",
        fallible: true,
        output_type: "unit",
    },
    // ── JSONL string variants (closes prior chat_members.rs gap) ─────────────
    PrimitiveEntry {
        name: "parse_jsonl_str",
        signature: "fn parse_jsonl_str<T: DeserializeOwned>(content: &str) -> Vec<T>",
        emit_template: "let {out}: Vec<{T}> = {content}.lines().filter(|l| !l.trim().is_empty()).filter_map(|l| serde_json::from_str(l.trim()).ok()).collect();",
        fallible: false,
        output_type: "Vec<T>",
    },
    PrimitiveEntry {
        name: "serialize_jsonl_line",
        signature: "fn serialize_jsonl_line<T: Serialize>(value: &T) -> Result<String>",
        emit_template: "let {out} = format!(\"{{}}\\n\", serde_json::to_string(&{value})?);",
        fallible: true,
        output_type: "string",
    },
    // ── Subprocess (partial closure of #tail-f-stream) ───────────────────────
    PrimitiveEntry {
        name: "run_subprocess",
        signature: "fn run_subprocess(program: &str, args: &[&str]) -> Result<std::process::Child>",
        emit_template: "let {out} = std::process::Command::new({program}).args({args}).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::null()).spawn()?;",
        fallible: true,
        output_type: "Child",
    },
    // ── Serde ────────────────────────────────────────────────────────────────
    PrimitiveEntry {
        name: "parse_yaml",
        signature: "fn parse_yaml<T: DeserializeOwned>(text: &str) -> Result<T>",
        emit_template: "let {out}: {T} = serde_yaml::from_str({text})?;",
        fallible: true,
        output_type: "T",
    },
    PrimitiveEntry {
        name: "parse_json",
        signature: "fn parse_json<T: DeserializeOwned>(text: &str) -> Result<T>",
        emit_template: "let {out}: {T} = serde_json::from_str({text})?;",
        fallible: true,
        output_type: "T",
    },
    PrimitiveEntry {
        name: "serialize_yaml",
        signature: "fn serialize_yaml<T: Serialize>(value: &T) -> Result<String>",
        emit_template: "let {out} = serde_yaml::to_string(&{value})?;",
        fallible: true,
        output_type: "string",
    },
    // ── Format ───────────────────────────────────────────────────────────────
    PrimitiveEntry {
        name: "format_template",
        signature: "fn format_template(template: &str, vars: &HashMap<&str, &str>) -> String",
        emit_template: "let {out} = {vars}.iter().fold({template}.to_string(), |acc, (k, v)| acc.replace(&format!(\"{{{{{}}}}}}\", k), v));",
        fallible: false,
        output_type: "string",
    },
    // ── Time ─────────────────────────────────────────────────────────────────
    PrimitiveEntry {
        name: "now",
        signature: "fn now() -> chrono::DateTime<chrono::Utc>",
        emit_template: "let {out} = chrono::Utc::now();",
        fallible: false,
        output_type: "DateTime",
    },
    // ── IO surface ───────────────────────────────────────────────────────────
    PrimitiveEntry {
        name: "tty_check",
        signature: "fn tty_check() -> bool",
        emit_template: "let {out} = atty::is(atty::Stream::Stdout);",
        fallible: false,
        output_type: "bool",
    },
    PrimitiveEntry {
        name: "print_stdout",
        signature: "fn print_stdout(text: &str)",
        emit_template: "println!(\"{{}}\", {text});",
        fallible: false,
        output_type: "unit",
    },
    // ── Generic ──────────────────────────────────────────────────────────────
    PrimitiveEntry {
        name: "call",
        signature: "fn call<T>(target: impl Fn(...) -> T, args: ...) -> Result<T>",
        emit_template: "let {out} = {target}({args})?;",
        fallible: true,
        output_type: "T",
    },
];

/// Look up a [`PrimitiveEntry`] by its [`PrimitiveKind`] discriminant.
///
/// Returns `None` if the kind is not in the MVP registry (deferred primitives
/// from the full spec vocabulary are not yet registered — see Step 2 sub-issues).
///
/// @spec projects/agentic-workflow/tech-design/surface/specs/mermaid-plus-primitive-vocabulary.md#schema
pub fn lookup(kind: &PrimitiveKind) -> Option<&'static PrimitiveEntry> {
    let name = kind_to_name(kind);
    REGISTRY.iter().find(|e| e.name == name)
}

/// Look up a [`PrimitiveEntry`] by its snake_case name string.
///
/// Used by the logic generator when a flowchart node carries `primitive: <name>`
/// as YAML — the YAML parser stores the name as `String` rather than the typed
/// enum to keep `content::logic` decoupled from `flowchart_plus::schema`.
///
/// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md (gap-blocker #flowchart-to-fn)
pub fn lookup_by_name(name: &str) -> Option<&'static PrimitiveEntry> {
    REGISTRY.iter().find(|e| e.name == name)
}

/// Substitute `{var}` placeholders in `template` with values from `bindings`.
///
/// Doubled braces `{{` / `}}` pass through as `format!`-style literal-brace
/// escapes for the emitted code. Unknown vars are preserved literally so a
/// missing binding surfaces as a compile error pointing at the exact placeholder.
///
/// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md (gap-blocker #flowchart-to-fn)
pub fn substitute_template(template: &str, bindings: &[(&str, &str)]) -> String {
    let chars: Vec<char> = template.chars().collect();
    let mut out = String::with_capacity(template.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '{' && i + 1 < chars.len() && chars[i + 1] == '{' {
            out.push_str("{{");
            i += 2;
            continue;
        }
        if c == '}' && i + 1 < chars.len() && chars[i + 1] == '}' {
            out.push_str("}}");
            i += 2;
            continue;
        }
        if c == '{' {
            let mut j = i + 1;
            while j < chars.len() && chars[j] != '}' {
                j += 1;
            }
            if j < chars.len() {
                let var_name: String = chars[i + 1..j].iter().collect();
                if let Some(&(_, val)) = bindings.iter().find(|(k, _)| k == &var_name) {
                    out.push_str(val);
                } else {
                    out.push('{');
                    out.push_str(&var_name);
                    out.push('}');
                }
                i = j + 1;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Classify a [`SectionType`] as prose (free-form markdown body allowed) vs
/// structural (must carry a fenced block or placeholder).
///
/// Prose types per the section-format-rule TD spec: Overview, Doc,
/// Requirements, TestPlan, Scenarios. All remaining variants are structural.
///
/// This is the registry-side definition of the prose-section taxonomy. The
/// previously hand-written `validate::rules::section_format::is_prose_section`
/// re-exports this constant so the policy lives in exactly one place.
///
/// @spec projects/agentic-workflow/tech-design/core/validate/section-format-rule.md#requirements
pub const fn is_prose_section(t: SectionType) -> bool {
    matches!(
        t,
        SectionType::Overview
            | SectionType::Doc
            | SectionType::Requirements
            | SectionType::TestPlan
            | SectionType::Scenarios
    )
}

/// Map a [`PrimitiveKind`] variant to its snake_case registry name string.
///
/// @spec projects/agentic-workflow/tech-design/surface/specs/mermaid-plus-primitive-vocabulary.md#schema
pub fn kind_to_name(kind: &PrimitiveKind) -> &'static str {
    match kind {
        PrimitiveKind::ReadFile => "read_file",
        PrimitiveKind::WriteFile => "write_file",
        PrimitiveKind::AppendFile => "append_file",
        PrimitiveKind::PathExists => "path_exists",
        PrimitiveKind::ParseJsonlStream => "parse_jsonl_stream",
        PrimitiveKind::AppendLineAtomic => "append_line_atomic",
        PrimitiveKind::ParseJsonlStr => "parse_jsonl_str",
        PrimitiveKind::SerializeJsonlLine => "serialize_jsonl_line",
        PrimitiveKind::RunSubprocess => "run_subprocess",
        PrimitiveKind::ParseYaml => "parse_yaml",
        PrimitiveKind::ParseJson => "parse_json",
        PrimitiveKind::SerializeYaml => "serialize_yaml",
        PrimitiveKind::FormatTemplate => "format_template",
        PrimitiveKind::Now => "now",
        PrimitiveKind::TtyCheck => "tty_check",
        PrimitiveKind::PrintStdout => "print_stdout",
        PrimitiveKind::Call => "call",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: registry completeness — all 12 MVP primitives are present

    #[test]
    fn test_registry_has_seventeen_entries() {
        // 12 MVP + 3 added by score-chat-jsonl-migration (parse_jsonl_stream,
        // append_line_atomic, run_subprocess) + 2 added by
        // enhancement-swap-handwrite-codegen-on-parse-channel-jsonl-seri
        // (parse_jsonl_str, serialize_jsonl_line)
        assert_eq!(REGISTRY.len(), 17);
    }

    /// REQ jsonl-str-primitives.md#tests T1 — parse_jsonl_str registered correctly
    #[test]
    fn test_lookup_parse_jsonl_str_returns_entry() {
        let entry =
            lookup(&PrimitiveKind::ParseJsonlStr).expect("parse_jsonl_str must be registered");
        assert_eq!(entry.name, "parse_jsonl_str");
        assert!(
            !entry.fallible,
            "string-input parse silently drops malformed lines"
        );
        assert_eq!(entry.output_type, "Vec<T>");
        assert!(
            entry.emit_template.contains(".lines()"),
            "must iterate &str via .lines(), got: {}",
            entry.emit_template
        );
        assert!(
            entry.emit_template.contains("serde_json::from_str"),
            "must parse each line via serde_json::from_str, got: {}",
            entry.emit_template
        );
        assert!(
            entry.emit_template.contains("filter_map"),
            "must drop parse errors via filter_map, got: {}",
            entry.emit_template
        );
        // Crucially: no File::open — this is the &str variant
        assert!(
            !entry.emit_template.contains("File::open"),
            "string variant must NOT open a file path, got: {}",
            entry.emit_template
        );
    }

    /// REQ jsonl-str-primitives.md#tests T2 — serialize_jsonl_line registered correctly
    #[test]
    fn test_lookup_serialize_jsonl_line_returns_entry() {
        let entry = lookup(&PrimitiveKind::SerializeJsonlLine)
            .expect("serialize_jsonl_line must be registered");
        assert_eq!(entry.name, "serialize_jsonl_line");
        assert!(entry.fallible, "serde_json::to_string can fail");
        assert_eq!(entry.output_type, "string");
        assert!(
            entry.emit_template.contains("serde_json::to_string"),
            "must serialize via serde_json::to_string, got: {}",
            entry.emit_template
        );
        assert!(
            entry.emit_template.contains("\\n"),
            "must append trailing newline for JSONL framing, got: {}",
            entry.emit_template
        );
        assert!(
            entry.emit_template.contains("?"),
            "must propagate serde error via ?, got: {}",
            entry.emit_template
        );
        // Crucially: no OpenOptions — this is the serialize-only variant
        assert!(
            !entry.emit_template.contains("OpenOptions"),
            "serialize-only variant must NOT open a file, got: {}",
            entry.emit_template
        );
    }

    #[test]
    fn test_lookup_run_subprocess_returns_entry() {
        let entry =
            lookup(&PrimitiveKind::RunSubprocess).expect("run_subprocess must be registered");
        assert_eq!(entry.name, "run_subprocess");
        assert!(entry.fallible, "spawn can fail");
        assert_eq!(entry.output_type, "Child");
        assert!(
            entry.emit_template.contains("Command::new"),
            "must emit Command::new spawn"
        );
        assert!(
            entry.emit_template.contains("Stdio::piped"),
            "must pipe stdout for caller streaming"
        );
    }

    #[test]
    fn test_lookup_parse_jsonl_stream_returns_entry() {
        let entry = lookup(&PrimitiveKind::ParseJsonlStream)
            .expect("parse_jsonl_stream must be registered");
        assert_eq!(entry.name, "parse_jsonl_stream");
        assert!(
            !entry.fallible,
            "JSONL parse swallows malformed lines, never errors"
        );
        assert_eq!(entry.output_type, "Vec<T>");
        assert!(
            entry.emit_template.contains("BufReader"),
            "must emit BufReader streaming parse"
        );
        assert!(
            entry.emit_template.contains("serde_json::from_str"),
            "must emit serde_json::from_str per line"
        );
    }

    #[test]
    fn test_lookup_append_line_atomic_returns_entry() {
        let entry = lookup(&PrimitiveKind::AppendLineAtomic)
            .expect("append_line_atomic must be registered");
        assert_eq!(entry.name, "append_line_atomic");
        assert!(entry.fallible);
        assert_eq!(entry.output_type, "unit");
        assert!(
            entry.emit_template.contains("OpenOptions"),
            "must emit O_APPEND open"
        );
        assert!(
            entry.emit_template.contains("serde_json::to_string"),
            "must emit JSON serialization"
        );
        assert!(
            entry.emit_template.contains(r#"\n"#) || entry.emit_template.contains("\\n"),
            "must emit trailing newline for JSONL framing"
        );
    }

    #[test]
    fn test_lookup_read_file_returns_entry() {
        let entry = lookup(&PrimitiveKind::ReadFile).expect("read_file must be registered");
        assert_eq!(entry.name, "read_file");
        assert!(entry.fallible);
        assert_eq!(entry.output_type, "string");
        assert!(entry.emit_template.contains("read_to_string"));
    }

    // REQ score-chat-jsonl-migration — substitute_template basic vars
    #[test]
    fn test_substitute_template_basic_vars() {
        let out = substitute_template(
            "let {out}: {T} = parse({path})?;",
            &[("out", "msgs"), ("T", "Vec<Msg>"), ("path", "PATH")],
        );
        assert_eq!(out, "let msgs: Vec<Msg> = parse(PATH)?;");
    }

    // REQ score-chat-jsonl-migration — preserves {{ }} format escapes
    #[test]
    fn test_substitute_template_preserves_double_brace_escapes() {
        let out = substitute_template("println!(\"{{}}\", {x});", &[("x", "42")]);
        assert_eq!(out, "println!(\"{{}}\", 42);");
    }

    // REQ score-chat-jsonl-migration — unknown var stays literal
    #[test]
    fn test_substitute_template_unknown_var_preserved() {
        let out = substitute_template("a {known} b {unknown} c", &[("known", "K")]);
        assert_eq!(out, "a K b {unknown} c");
    }

    // REQ score-chat-jsonl-migration — lookup_by_name finds entries
    #[test]
    fn test_lookup_by_name_finds_jsonl_primitives() {
        assert!(lookup_by_name("parse_jsonl_stream").is_some());
        assert!(lookup_by_name("append_line_atomic").is_some());
        assert!(lookup_by_name("parse_jsonl_str").is_some());
        assert!(lookup_by_name("serialize_jsonl_line").is_some());
        assert!(lookup_by_name("run_subprocess").is_some());
        assert!(lookup_by_name("does_not_exist").is_none());
    }

    #[test]
    fn test_lookup_path_exists_is_infallible() {
        let entry = lookup(&PrimitiveKind::PathExists).expect("path_exists must be registered");
        assert!(!entry.fallible, "path_exists must be infallible");
        assert_eq!(entry.output_type, "bool");
    }

    #[test]
    fn test_lookup_write_file_is_fallible() {
        let entry = lookup(&PrimitiveKind::WriteFile).expect("write_file must be registered");
        assert!(entry.fallible);
        assert_eq!(entry.output_type, "unit");
    }

    #[test]
    fn test_lookup_parse_yaml_generic_output() {
        let entry = lookup(&PrimitiveKind::ParseYaml).expect("parse_yaml must be registered");
        assert_eq!(entry.output_type, "T");
        assert!(entry.fallible);
    }

    #[test]
    fn test_lookup_now_is_infallible() {
        let entry = lookup(&PrimitiveKind::Now).expect("now must be registered");
        assert!(!entry.fallible);
        assert_eq!(entry.output_type, "DateTime");
    }

    #[test]
    fn test_lookup_tty_check_is_infallible_bool() {
        let entry = lookup(&PrimitiveKind::TtyCheck).expect("tty_check must be registered");
        assert!(!entry.fallible);
        assert_eq!(entry.output_type, "bool");
    }

    #[test]
    fn test_lookup_print_stdout_is_infallible_unit() {
        let entry = lookup(&PrimitiveKind::PrintStdout).expect("print_stdout must be registered");
        assert!(!entry.fallible);
        assert_eq!(entry.output_type, "unit");
    }

    #[test]
    fn test_lookup_call_is_fallible_generic() {
        let entry = lookup(&PrimitiveKind::Call).expect("call must be registered");
        assert!(entry.fallible);
        assert_eq!(entry.output_type, "T");
    }

    #[test]
    fn test_all_registry_names_are_unique() {
        let mut names: Vec<&str> = REGISTRY.iter().map(|e| e.name).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(before, names.len(), "registry contains duplicate names");
    }

    #[test]
    fn test_all_registry_entries_have_non_empty_emit_template() {
        for entry in REGISTRY {
            assert!(
                !entry.emit_template.is_empty(),
                "entry '{}' has empty emit_template",
                entry.name
            );
        }
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/primitive_registry.rs
    action: modify
    section: source
    impl_mode: codegen
    description: "Source template owns the full primitive registry module."
```
