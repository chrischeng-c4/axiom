// SPEC-MANAGED: projects/guard/tech-design/semantic/source/projects-guard-src-config-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `guard.toml` config — the agent-first knob home. A bare `guard scan` reads
//! the policy profile, persistence, and scan paths from here, so the common
//! path is zero-args. Per-run overrides (`--profile`, `--no-persist`) stay on
//! the CLI.
//!
//! Unknown keys are ignored, so this coexists with an aw-managed `AW-EC-TOOL`
//! manifest block in the same `guard.toml`.

use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::scan::PolicyProfile;

/// Parsed `guard.toml` launcher config (the launcher keys only).
#[derive(Debug, Clone, Default, Deserialize)]
/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-config-rs.md#source
pub struct GuardConfig {
    /// Policy profile spelled `baseline-static` | `security-lint` | `strict`.
    #[serde(default)]
    pub profile: Option<String>,
    /// Skip persisting `.guard/last-report.json` by default.
    #[serde(default)]
    pub no_persist: Option<bool>,
    /// File or directory targets to scan when no positional path is given.
    #[serde(default)]
    pub paths: Vec<PathBuf>,
}

/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-config-rs.md#source
impl GuardConfig {
    /// Load `guard.toml` from `dir`. Absent file or parse error => defaults.
    /// Unknown keys (an aw-generated `AW-EC-TOOL` block) are ignored.
    pub fn load_from(dir: &Path) -> GuardConfig {
        std::fs::read_to_string(dir.join("guard.toml"))
            .ok()
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_default()
    }

    /// The configured policy profile, if any (and recognized).
    pub fn profile(&self) -> Option<PolicyProfile> {
        self.profile
            .as_deref()
            .and_then(PolicyProfile::from_config_str)
    }

    /// The scan targets: explicit `paths`, else `fallback`.
    pub fn scan_paths(&self, fallback: &str) -> Vec<PathBuf> {
        if self.paths.is_empty() {
            vec![PathBuf::from(fallback)]
        } else {
            self.paths.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_when_absent() {
        let config = GuardConfig::load_from(Path::new("/nonexistent-guard-xyz"));
        assert!(config.profile().is_none());
        assert!(config.no_persist.is_none());
        assert_eq!(config.scan_paths("."), vec![PathBuf::from(".")]);
    }

    #[test]
    fn reads_launcher_keys_and_ignores_aw_block() {
        let dir = std::env::temp_dir().join(format!("guard-cfg-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("guard.toml"),
            // aw-managed keys + launcher keys coexist in one file:
            "project = \"guard\"\nsource_contract = \"x\"\nprofile = \"security-lint\"\nno_persist = true\npaths = [\"src\", \"guard-cli\"]\n",
        )
        .unwrap();

        let config = GuardConfig::load_from(&dir);
        assert_eq!(config.profile(), Some(PolicyProfile::SecurityLint));
        assert_eq!(config.no_persist, Some(true));
        assert_eq!(
            config.scan_paths("."),
            vec![PathBuf::from("src"), PathBuf::from("guard-cli")]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_profile_string_is_ignored() {
        let dir = std::env::temp_dir().join(format!("guard-cfg-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("guard.toml"), "profile = \"nonsense\"\n").unwrap();
        let config = GuardConfig::load_from(&dir);
        assert!(config.profile().is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
// CODEGEN-END
