// PEP 723 typed view (Tick 116).
//
// The existing `pep723` module returns dependencies as raw `Vec<String>`
// — verbatim PEP 508 lines preserved for downstream consumers. This
// sibling module layers on a typed view that routes each line through
// `requirement_string::Requirement::parse`, so callers (resolver,
// installer, `uv run`-equivalent) see structurally split
// name + extras + specifier + marker exactly the way they do for
// pyproject.toml dependencies (see `pep621.rs`).
//
// The original `pep723::parse_pep723` is left untouched — its many
// existing callers keep their string-typed surface, while new
// consumers opt into typing through `TypedScriptMetadata`.

use crate::pkgmanage::pkgmgr::pep723::{parse_pep723, ScriptMetadata};
use crate::pkgmanage::pkgmgr::requirement_string::Requirement;
use crate::pkgmanage::pkgmgr::types::IndexError;

/// Typed view of a PEP 723 `# /// script` metadata block.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TypedScriptMetadata {
    /// Raw `requires-python` clause text. Downstream callers feed this
    /// into [`crate::pkgmanage::pkgmgr::specifier_set::SpecifierSet`] when
    /// they need full PEP 440 semantics.
    pub requires_python: Option<String>,
    /// Parsed PEP 508 dependency lines.
    pub dependencies: Vec<Requirement>,
}

impl TypedScriptMetadata {
    /// Locate the `# /// script` block in `source` and return a typed
    /// view. Returns `Ok(None)` when the script has no PEP 723 block
    /// (PEP 723 inactive — caller should fall back to interpreter-only
    /// execution). Surfaces any underlying `pep723::parse_pep723` error.
    pub fn from_source(source: &str) -> Result<Option<Self>, IndexError> {
        let Some(meta) = parse_pep723(source)? else {
            return Ok(None);
        };
        Self::from_metadata(&meta).map(Some)
    }

    /// Upgrade an already-parsed [`ScriptMetadata`] into a typed view.
    /// Useful for callers that need both the verbatim raw form (for
    /// round-trip writes via `pep723::upsert_pep723`) and the typed form.
    pub fn from_metadata(meta: &ScriptMetadata) -> Result<Self, IndexError> {
        let mut dependencies = Vec::with_capacity(meta.dependencies.len());
        for line in &meta.dependencies {
            dependencies.push(Requirement::parse(line)?);
        }
        Ok(Self {
            requires_python: meta.requires_python_raw.clone(),
            dependencies,
        })
    }

    /// True when no dependencies and no `requires-python` were declared.
    pub fn is_empty(&self) -> bool {
        self.requires_python.is_none() && self.dependencies.is_empty()
    }

    /// Convenience: look up a dependency by PEP 503-normalized name.
    pub fn find(&self, name: &str) -> Option<&Requirement> {
        let normalized = crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize(name);
        self.dependencies.iter().find(|r| r.name == normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(src: &str) -> TypedScriptMetadata {
        TypedScriptMetadata::from_source(src)
            .unwrap_or_else(|e| panic!("parse failed: {e:?}"))
            .unwrap_or_else(|| panic!("expected PEP 723 block to be present"))
    }

    #[test]
    fn returns_none_when_no_pep723_block() {
        let src = "print('hi')\n";
        assert!(TypedScriptMetadata::from_source(src).unwrap().is_none());
    }

    #[test]
    fn parses_minimal_block() {
        let src = r#"
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
print('hi')
"#;
        let t = typed(src);
        assert_eq!(t.requires_python.as_deref(), Some(">=3.10"));
        assert!(t.dependencies.is_empty());
    }

    #[test]
    fn routes_dependencies_through_requirement_string() {
        let src = r#"
# /// script
# dependencies = [
#   "requests>=2.31",
#   "Django[argon2]>=4.2",
# ]
# ///
print('hi')
"#;
        let t = typed(src);
        assert_eq!(t.dependencies.len(), 2);
        assert_eq!(t.dependencies[0].name, "requests");
        assert_eq!(t.dependencies[0].specifier.as_deref(), Some(">=2.31"));
        assert_eq!(t.dependencies[1].name, "django");
        assert!(t.dependencies[1].extras.contains("argon2"));
    }

    #[test]
    fn find_uses_pep503_normalization() {
        let src = r#"
# /// script
# dependencies = ["Zope.Interface>=5"]
# ///
"#;
        let t = typed(src);
        assert!(t.find("zope-interface").is_some());
        assert!(t.find("Zope.Interface").is_some());
        assert!(t.find("zope_interface").is_some());
        assert!(t.find("nonexistent").is_none());
    }

    #[test]
    fn is_empty_when_no_constraints_declared() {
        let src = r#"
# /// script
# ///
"#;
        let t = typed(src);
        assert!(t.is_empty());
    }

    #[test]
    fn surfaces_requirement_parse_errors() {
        let src = r#"
# /// script
# dependencies = ["[totally broken"]
# ///
"#;
        let err = TypedScriptMetadata::from_source(src).unwrap_err();
        let msg = format!("{err:?}");
        // Either the name-character validation or the bracket-balance
        // check fires — both are acceptable.
        assert!(msg.contains("letter or digit") || msg.contains("name"));
    }

    #[test]
    fn surfaces_underlying_pep723_errors() {
        // Unterminated block — must bubble up from parse_pep723.
        let src = r#"
# /// script
# dependencies = []
"#;
        assert!(TypedScriptMetadata::from_source(src).is_err());
    }

    #[test]
    fn from_metadata_upgrades_existing_struct() {
        let raw = ScriptMetadata {
            requires_python: None,
            requires_python_raw: Some(">=3.11".to_string()),
            dependencies: vec!["pydantic>=2".to_string(), "rich".to_string()],
        };
        let t = TypedScriptMetadata::from_metadata(&raw).unwrap();
        assert_eq!(t.requires_python.as_deref(), Some(">=3.11"));
        assert_eq!(t.dependencies.len(), 2);
        assert_eq!(t.dependencies[0].name, "pydantic");
        assert_eq!(t.dependencies[1].name, "rich");
        assert!(t.dependencies[1].specifier.is_none());
    }

    #[test]
    fn realistic_uv_run_script() {
        // This is the shape of script that `uv run script.py` consumes —
        // a top-of-file PEP 723 block + ordinary Python below.
        let src = r#"#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "httpx>=0.27",
#   "rich",
#   "typer[all]>=0.12 ; python_version >= '3.10'",
# ]
# ///

import httpx
from rich import print
import typer

app = typer.Typer()

@app.command()
def main(name: str = "world"):
    print(f"hello, {name}")

if __name__ == "__main__":
    app()
"#;
        let t = typed(src);
        assert_eq!(t.requires_python.as_deref(), Some(">=3.11"));
        assert_eq!(t.dependencies.len(), 3);
        assert_eq!(t.dependencies[0].name, "httpx");
        assert_eq!(t.dependencies[2].name, "typer");
        assert!(t.dependencies[2].extras.contains("all"));
        assert_eq!(
            t.dependencies[2].marker.as_deref(),
            Some("python_version >= '3.10'"),
        );
        assert_eq!(t.dependencies[2].specifier.as_deref(), Some(">=0.12"));
    }

    #[test]
    fn realistic_uv_run_script_marker_is_preserved() {
        // Same shape as above but a clean marker assertion (the test
        // above intentionally exercises the convoluted form too).
        let src = r#"# /// script
# dependencies = ["typer ; python_version >= '3.10'"]
# ///
"#;
        let t = typed(src);
        assert_eq!(
            t.dependencies[0].marker.as_deref(),
            Some("python_version >= '3.10'"),
        );
    }
}
