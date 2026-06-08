// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
// CODEGEN-BEGIN

//! `FixtureManifest` — parses the JSDoc-style `@fixture` frontmatter block
//! out of a TSX fixture file.
//!
//! Per spec R1 and the §Changes entry for `manifest.rs`:
//! ```text
//! /** @fixture { "name": "mui-button", "ime": false, "tab_count": 32 } */
//! ```

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// @spec parity-dom-reference-runner.md#Dependency (FixtureManifest)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FixtureManifest {
    pub name: String,
    #[serde(default)]
    pub ime: bool,
    #[serde(default = "default_tab_count")]
    pub tab_count: u32,
}

fn default_tab_count() -> u32 {
    32
}

/// @spec parity-dom-reference-runner.md#Changes (manifest.rs)
#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("fixture frontmatter block `/** @fixture ... */` not found")]
    Missing,
    #[error("fixture frontmatter JSON parse failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("fixture name `{0}` is not kebab-case")]
    BadName(String),
    #[error("fixture tab_count {0} out of allowed range 0..=256")]
    BadTabCount(u32),
    #[error("fixture file IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
impl FixtureManifest {
    /// @spec parity-dom-reference-runner.md#Changes (manifest.rs)
    ///
    /// Parse a fixture file's frontmatter block. The block must be the
    /// JSDoc form `/** @fixture { ... } */` (single- or multi-line).
    pub fn from_file(path: &Path) -> Result<Self, ManifestError> {
        let src = std::fs::read_to_string(path)?;
        Self::from_source(&src)
    }

    /// @spec parity-dom-reference-runner.md#Changes (manifest.rs)
    pub fn from_source(src: &str) -> Result<Self, ManifestError> {
        let json = extract_frontmatter(src).ok_or(ManifestError::Missing)?;
        let manifest: FixtureManifest = serde_json::from_str(&json)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// @spec parity-dom-reference-runner.md#Changes (manifest.rs)
    pub fn validate(&self) -> Result<(), ManifestError> {
        if !is_kebab_case(&self.name) {
            return Err(ManifestError::BadName(self.name.clone()));
        }
        if self.tab_count > 256 {
            return Err(ManifestError::BadTabCount(self.tab_count));
        }
        Ok(())
    }
}

fn is_kebab_case(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !s.starts_with('-')
        && !s.ends_with('-')
}

fn extract_frontmatter(src: &str) -> Option<String> {
    // Find `/** ... @fixture ... */`. Tolerate leading whitespace + multi-line.
    let start_marker = "@fixture";
    let start_idx = src.find(start_marker)?;
    // The opening `/**` must appear somewhere before `@fixture`, with only
    // whitespace + `*` continuation chars between them.
    let before = &src[..start_idx];
    let open_idx = before.rfind("/**")?;
    let between = &before[open_idx + 3..];
    if !between.chars().all(|c| c.is_whitespace() || c == '*') {
        return None;
    }
    let after_marker = &src[start_idx + start_marker.len()..];
    // The block ends at the next `*/`.
    let end_idx = after_marker.find("*/")?;
    let body = &after_marker[..end_idx];
    // Strip leading `*` line continuations and surrounding whitespace.
    let cleaned: String = body
        .lines()
        .map(|line| line.trim().trim_start_matches('*').trim())
        .collect::<Vec<_>>()
        .join(" ");
    let cleaned = cleaned.trim();
    // Expect cleaned to begin with `{` and contain JSON object.
    let json_start = cleaned.find('{')?;
    let json_end = cleaned.rfind('}')?;
    if json_end < json_start {
        return None;
    }
    Some(cleaned[json_start..=json_end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_inline_frontmatter() {
        let src = r#"/** @fixture { "name": "mui-button", "ime": false, "tab_count": 8 } */
import { Button } from '@mui/material';
export default function MuiButton() { return <Button>Click</Button>; }
"#;
        let m = FixtureManifest::from_source(src).unwrap();
        assert_eq!(m.name, "mui-button");
        assert!(!m.ime);
        assert_eq!(m.tab_count, 8);
    }

    #[test]
    fn parses_multiline_frontmatter() {
        let src = r#"/**
 * @fixture {
 *   "name": "mui-text-field",
 *   "ime": true,
 *   "tab_count": 16
 * }
 */
"#;
        let m = FixtureManifest::from_source(src).unwrap();
        assert_eq!(m.name, "mui-text-field");
        assert!(m.ime);
        assert_eq!(m.tab_count, 16);
    }

    #[test]
    fn missing_frontmatter_errors() {
        let err = FixtureManifest::from_source("export default () => null;").unwrap_err();
        assert!(matches!(err, ManifestError::Missing));
    }

    #[test]
    fn bad_kebab_case_rejected() {
        let src = r#"/** @fixture { "name": "MuiButton" } */"#;
        let err = FixtureManifest::from_source(src).unwrap_err();
        assert!(matches!(err, ManifestError::BadName(_)));
    }

    #[test]
    fn tab_count_over_max_rejected() {
        let src = r#"/** @fixture { "name": "x", "tab_count": 9999 } */"#;
        let err = FixtureManifest::from_source(src).unwrap_err();
        assert!(matches!(err, ManifestError::BadTabCount(_)));
    }

    #[test]
    fn defaults_tab_count_to_32() {
        let src = r#"/** @fixture { "name": "x" } */"#;
        let m = FixtureManifest::from_source(src).unwrap();
        assert_eq!(m.tab_count, 32);
        assert!(!m.ime);
    }
}
// CODEGEN-END
