// `uv init`-equivalent pyproject.toml scaffolding (Tick 31).
//
// uv's `init` writes a minimal but well-formed pyproject.toml so the
// user can immediately run `uv add foo` against it. The shape:
//
//   [project]
//   name = "<normalized>"
//   version = "0.1.0"
//   description = "Add your description here"
//   requires-python = ">=3.11"
//   readme = "README.md"
//   license = { text = "MIT" }         # optional
//   dependencies = []
//
//   [tool.uv]                          # only when relevant flags are set
//   <managed-by mamba in our case>
//
// This module:
//   * accepts a `ScaffoldOptions` describing the new project,
//   * validates the name (PEP 503 normalize roundtrip),
//   * returns a TOML *string*. The caller decides whether to write to
//     disk, print to stdout, or pipe somewhere else.
//
// No filesystem side effects from this module. uv-the-CLI handles the
// write; we keep this layer testable without temp dirs.

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Caller-supplied scaffold knobs. Sensible defaults are filled in by
/// `ScaffoldOptions::for_name(name)` so the common case is one line.
#[derive(Debug, Clone)]
pub struct ScaffoldOptions {
    /// Project name. Will be PEP 503-normalized when written.
    pub name: String,
    /// Initial version. uv defaults to `0.1.0`; we follow.
    pub version: String,
    /// `requires-python` PEP 440 specifier. Defaults to ">=3.11" to
    /// match uv's modern-default behavior.
    pub requires_python: String,
    /// One-line description. uv uses "Add your description here".
    pub description: String,
    /// Optional README filename. Setting `None` omits the field.
    pub readme: Option<String>,
    /// Optional `license = { text = ... }` value.
    pub license: Option<String>,
    /// Optional author entries. Each is rendered as
    /// `{ name = "...", email = "..." }`. Email is optional per entry.
    pub authors: Vec<AuthorEntry>,
    /// When true, append an empty `[tool.mamba]` table as a marker so
    /// `mamba` knows this project is managed by it.
    pub add_tool_marker: bool,
}

#[derive(Debug, Clone)]
pub struct AuthorEntry {
    pub name: String,
    pub email: Option<String>,
}

impl ScaffoldOptions {
    /// One-line constructor for the common case. Apply additional
    /// tweaks by mutating fields on the returned struct.
    pub fn for_name(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "0.1.0".to_string(),
            requires_python: ">=3.11".to_string(),
            description: "Add your description here".to_string(),
            readme: Some("README.md".to_string()),
            license: None,
            authors: Vec::new(),
            add_tool_marker: false,
        }
    }
}

/// Validate the project name. PEP 503 normalize must round-trip, and
/// the normalized form must be non-empty.
pub fn validate_project_name(name: &str) -> Result<String, IndexError> {
    if name.trim().is_empty() {
        return Err(IndexError::ParseError {
            url: "<pyproject.toml scaffold>".into(),
            detail: "project name must not be empty".into(),
        });
    }
    // PEP 503: lowercase + collapse runs of [-_.] to a single '-'.
    let lowered: String = name.chars().map(|c| c.to_ascii_lowercase()).collect();
    let mut normalized = String::with_capacity(lowered.len());
    let mut last_sep = false;
    for c in lowered.chars() {
        let is_sep = c == '-' || c == '_' || c == '.';
        if is_sep {
            if !last_sep {
                normalized.push('-');
            }
            last_sep = true;
        } else {
            normalized.push(c);
            last_sep = false;
        }
    }
    let normalized = normalized.trim_matches('-').to_string();
    if normalized.is_empty() {
        return Err(IndexError::ParseError {
            url: "<pyproject.toml scaffold>".into(),
            detail: format!("project name {name:?} normalizes to empty (only separators)"),
        });
    }
    // Disallow characters outside the PEP 508 identifier alphabet.
    // PEP 508 names: [A-Za-z0-9._-], but after normalize only
    // [a-z0-9-] survives. Anything else is a hard error at authoring.
    for c in normalized.chars() {
        if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err(IndexError::ParseError {
                url: "<pyproject.toml scaffold>".into(),
                detail: format!(
                    "project name {name:?} contains invalid character {c:?}; \
                     allowed: ASCII letters, digits, '-', '_', '.'"
                ),
            });
        }
    }
    Ok(normalized)
}

/// Render `ScaffoldOptions` as a pyproject.toml string. Deterministic
/// output for diffing in tests.
pub fn render_pyproject(opts: &ScaffoldOptions) -> Result<String, IndexError> {
    let name = validate_project_name(&opts.name)?;
    if opts.version.trim().is_empty() {
        return Err(IndexError::ParseError {
            url: "<pyproject.toml scaffold>".into(),
            detail: "version must not be empty".into(),
        });
    }
    if opts.requires_python.trim().is_empty() {
        return Err(IndexError::ParseError {
            url: "<pyproject.toml scaffold>".into(),
            detail: "requires-python must not be empty".into(),
        });
    }

    let mut out = String::new();
    out.push_str("[project]\n");
    out.push_str(&format!("name = {}\n", toml_string(&name)));
    out.push_str(&format!("version = {}\n", toml_string(&opts.version)));
    out.push_str(&format!(
        "description = {}\n",
        toml_string(&opts.description)
    ));
    out.push_str(&format!(
        "requires-python = {}\n",
        toml_string(&opts.requires_python)
    ));
    if let Some(readme) = &opts.readme {
        out.push_str(&format!("readme = {}\n", toml_string(readme)));
    }
    if let Some(license) = &opts.license {
        out.push_str(&format!(
            "license = {{ text = {} }}\n",
            toml_string(license)
        ));
    }
    if !opts.authors.is_empty() {
        out.push_str("authors = [\n");
        for a in &opts.authors {
            if let Some(email) = &a.email {
                out.push_str(&format!(
                    "    {{ name = {}, email = {} }},\n",
                    toml_string(&a.name),
                    toml_string(email)
                ));
            } else {
                out.push_str(&format!("    {{ name = {} }},\n", toml_string(&a.name)));
            }
        }
        out.push_str("]\n");
    }
    out.push_str("dependencies = []\n");

    if opts.add_tool_marker {
        out.push_str("\n[tool.mamba]\n");
        // Currently empty — presence is the marker. Future configuration
        // (e.g. index-url, resolution mode) gets parsed from here.
    }

    Ok(out)
}

/// Quote a TOML string literal. We use basic strings (double-quoted)
/// and escape `"` and `\`. uv emits the same form; this keeps the diff
/// stable when comparing scaffolds.
fn toml_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_accepts_simple_name() {
        assert_eq!(validate_project_name("Requests").unwrap(), "requests");
    }

    #[test]
    fn validate_collapses_separators() {
        assert_eq!(
            validate_project_name("My__Pkg.Name").unwrap(),
            "my-pkg-name"
        );
        assert_eq!(
            validate_project_name("--leading--trail--").unwrap(),
            "leading-trail"
        );
    }

    #[test]
    fn validate_rejects_empty() {
        let err = validate_project_name("").unwrap_err();
        assert!(format!("{err}").contains("must not be empty"));
    }

    #[test]
    fn validate_rejects_only_separators() {
        let err = validate_project_name("---").unwrap_err();
        assert!(format!("{err}").contains("normalizes to empty"));
    }

    #[test]
    fn validate_rejects_unicode_or_punct() {
        let err = validate_project_name("café").unwrap_err();
        assert!(format!("{err}").contains("invalid character"));
    }

    #[test]
    fn render_basic_for_name() {
        let opts = ScaffoldOptions::for_name("Demo");
        let toml = render_pyproject(&opts).unwrap();
        assert!(toml.contains("name = \"demo\""));
        assert!(toml.contains("version = \"0.1.0\""));
        assert!(toml.contains("requires-python = \">=3.11\""));
        assert!(toml.contains("readme = \"README.md\""));
        assert!(toml.contains("dependencies = []"));
        assert!(!toml.contains("[tool.mamba]"));
        assert!(!toml.contains("license"));
    }

    #[test]
    fn render_includes_license_when_set() {
        let mut opts = ScaffoldOptions::for_name("demo");
        opts.license = Some("MIT".to_string());
        let toml = render_pyproject(&opts).unwrap();
        assert!(toml.contains("license = { text = \"MIT\" }"));
    }

    #[test]
    fn render_emits_author_block() {
        let mut opts = ScaffoldOptions::for_name("demo");
        opts.authors.push(AuthorEntry {
            name: "Ada".into(),
            email: Some("ada@example.test".into()),
        });
        opts.authors.push(AuthorEntry {
            name: "Solo".into(),
            email: None,
        });
        let toml = render_pyproject(&opts).unwrap();
        assert!(toml.contains("authors = ["));
        assert!(toml.contains("{ name = \"Ada\", email = \"ada@example.test\" },"));
        assert!(toml.contains("{ name = \"Solo\" },"));
    }

    #[test]
    fn render_adds_tool_marker_when_requested() {
        let mut opts = ScaffoldOptions::for_name("demo");
        opts.add_tool_marker = true;
        let toml = render_pyproject(&opts).unwrap();
        assert!(toml.ends_with("[tool.mamba]\n"));
    }

    #[test]
    fn render_normalizes_name_in_output() {
        let opts = ScaffoldOptions::for_name("My_Pkg.Name");
        let toml = render_pyproject(&opts).unwrap();
        assert!(toml.contains("name = \"my-pkg-name\""));
    }

    #[test]
    fn render_quotes_special_chars_in_description() {
        let mut opts = ScaffoldOptions::for_name("demo");
        opts.description = "It's \"quoted\" and \\ has a backslash".into();
        let toml = render_pyproject(&opts).unwrap();
        assert!(toml.contains("description = \"It's \\\"quoted\\\" and \\\\ has a backslash\""));
    }

    #[test]
    fn render_rejects_empty_version() {
        let mut opts = ScaffoldOptions::for_name("demo");
        opts.version = "".into();
        let err = render_pyproject(&opts).unwrap_err();
        assert!(format!("{err}").contains("version must not be empty"));
    }

    #[test]
    fn render_rejects_empty_requires_python() {
        let mut opts = ScaffoldOptions::for_name("demo");
        opts.requires_python = "".into();
        let err = render_pyproject(&opts).unwrap_err();
        assert!(format!("{err}").contains("requires-python must not be empty"));
    }

    #[test]
    fn render_output_parses_as_valid_toml() {
        let mut opts = ScaffoldOptions::for_name("Demo-Pkg");
        opts.license = Some("Apache-2.0".into());
        opts.authors.push(AuthorEntry {
            name: "Ada".into(),
            email: Some("ada@example.test".into()),
        });
        opts.add_tool_marker = true;
        let rendered = render_pyproject(&opts).unwrap();
        // Round-trip through the `toml` crate to catch syntactic
        // mistakes (missing commas, unescaped quotes, etc.).
        let parsed: toml::Value = rendered.parse().expect("rendered must be valid TOML");
        let project = parsed
            .get("project")
            .and_then(|v| v.as_table())
            .expect("missing [project] table");
        assert_eq!(
            project.get("name").and_then(|v| v.as_str()),
            Some("demo-pkg")
        );
        assert!(parsed.get("tool").is_some(), "tool table missing");
    }
}
