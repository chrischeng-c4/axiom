// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
// CODEGEN-BEGIN
//! `dist/jet-target.json` — single small JSON file emitted alongside
//! every successful build, consumed by downstream packagers (Tauri
//! shim from #1242, ratatui launcher from #1241) and CI dashboards.
//!
//! @spec .aw/tech-design/projects/jet/logic/multi-target/build-targets.md
//! (Slice 3: manifest schema — single source of truth lives there;
//! this module mirrors it).

use crate::build_target::BuildTarget;
use anyhow::{Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// Hard-coded; bump on incompatible changes. Slice-3 ships v1.
pub const SCHEMA_VERSION: u32 = 1;
pub const TSX_LOWERING_STRICT: &str = "strict";
pub const TSX_LOWERING_COMPATIBILITY: &str = "compatibility";

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
#[derive(Debug, Serialize)]
pub struct Manifest {
    pub schema_version: u32,
    pub target: &'static str,
    /// Resolves via the `target-profiles.yaml` `inherits:` chain. For
    /// `desktop` this is `web`; for the others it equals `target`.
    pub profile_target: &'static str,
    /// Hint for downstream tooling — `null` for web, `"tauri"` for
    /// desktop, `"ratatui"` for TUI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_for: Option<&'static str>,
    pub artifact: Artifact,
    pub build: Build,
    pub source: Source,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
#[derive(Debug, Serialize)]
pub struct Artifact {
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wasm_path: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_path: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_path: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_path: Option<&'static str>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
#[derive(Debug, Serialize)]
pub struct Build {
    pub mode: &'static str,
    pub rustc_target: &'static str,
    pub cargo_features: Vec<String>,
    pub tsx_lowering: &'static str,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
#[derive(Debug, Serialize)]
pub struct Source {
    pub entry: String,
    pub root_component: String,
    /// `sha256:<hex>` of `jet.config.toml` so manifest diffs are
    /// reproducible.
    pub jet_config_hash: String,
}

/// Inputs the wasm pipeline already has on hand by the time it
/// finishes — collected into one struct to keep `build_with_profile`
/// readable.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
pub struct ManifestInputs<'a> {
    pub target: BuildTarget,
    pub profile_mode: &'static str,
    pub entry: &'a str,
    pub root_component: &'a str,
    pub jet_config_path: &'a Path,
    pub cargo_features: Vec<String>,
    pub tsx_lowering: &'static str,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
impl Manifest {
    pub fn build(inputs: ManifestInputs<'_>) -> Result<Self> {
        let bytes = fs::read(inputs.jet_config_path).with_context(|| {
            format!(
                "reading {} for manifest hash",
                inputs.jet_config_path.display()
            )
        })?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hex = hex_lower(&hasher.finalize());
        Ok(Self {
            schema_version: SCHEMA_VERSION,
            target: inputs.target.as_str(),
            profile_target: profile_target_for(inputs.target),
            package_for: package_for(inputs.target),
            artifact: artifact_for(inputs.target),
            build: Build {
                mode: inputs.profile_mode,
                rustc_target: rustc_target_for(inputs.target),
                cargo_features: inputs.cargo_features,
                tsx_lowering: inputs.tsx_lowering,
            },
            source: Source {
                entry: inputs.entry.to_string(),
                root_component: inputs.root_component.to_string(),
                jet_config_hash: format!("sha256:{hex}"),
            },
        })
    }
}

/// Walks the `inherits:` chain — for desktop this is `web`; the rest
/// equal their own target. Hard-coded to mirror the YAML; if a fourth
/// target lands, extend the YAML and update this map together.
fn profile_target_for(t: BuildTarget) -> &'static str {
    match t {
        BuildTarget::Web => "web",
        BuildTarget::Desktop => "web",
        BuildTarget::Tui => "tui",
    }
}

fn package_for(t: BuildTarget) -> Option<&'static str> {
    match t {
        BuildTarget::Web => None,
        BuildTarget::Desktop => Some("tauri"),
        BuildTarget::Tui => Some("ratatui"),
    }
}

fn artifact_for(t: BuildTarget) -> Artifact {
    if t.produces_wasm() {
        Artifact {
            kind: "wasm",
            wasm_path: Some("app_bg.wasm"),
            boot_path: Some("boot.js"),
            host_path: Some("jet-host.js"),
            html_path: Some("index.html"),
        }
    } else {
        Artifact {
            kind: "native-bin",
            wasm_path: None,
            boot_path: None,
            host_path: None,
            html_path: None,
        }
    }
}

fn rustc_target_for(t: BuildTarget) -> &'static str {
    if t.produces_wasm() {
        "wasm32-unknown-unknown"
    } else {
        // TUI native binary; the host triple is filled in by #1241
        // when that pipeline lands. Keep "host" as a placeholder so
        // the schema is stable across slices.
        "host"
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Serialize + write `<dist>/jet-target.json`. Pretty-printed so
/// manifest diffs in CI are readable.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
pub fn write(dist: &Path, manifest: &Manifest) -> Result<()> {
    let json = serde_json::to_string_pretty(manifest).context("serializing jet-target.json")?;
    fs::write(dist.join("jet-target.json"), json).context("writing jet-target.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn fixture_inputs<'a>(target: BuildTarget, cfg_path: &'a Path) -> ManifestInputs<'a> {
        ManifestInputs {
            target,
            profile_mode: "release",
            entry: "src/index.tsx",
            root_component: "App",
            jet_config_path: cfg_path,
            cargo_features: vec![
                "jet-multi-target/target-web".into(),
                "jet-multi-target/web".into(),
            ],
            tsx_lowering: TSX_LOWERING_STRICT,
        }
    }

    fn write_cfg(dir: &Path, contents: &str) -> std::path::PathBuf {
        let p = dir.join("jet.config.toml");
        fs::write(&p, contents).unwrap();
        p
    }

    #[test]
    fn web_manifest_has_wasm_artifact_and_no_package_for() {
        let tmp = TempDir::new().unwrap();
        let cfg = write_cfg(tmp.path(), "[wasm]\nentry = 'src/index.tsx'\n");
        let m = Manifest::build(fixture_inputs(BuildTarget::Web, &cfg)).unwrap();
        assert_eq!(m.target, "web");
        assert_eq!(m.profile_target, "web");
        assert!(m.package_for.is_none());
        assert_eq!(m.artifact.kind, "wasm");
        assert_eq!(m.artifact.wasm_path, Some("app_bg.wasm"));
        assert_eq!(m.artifact.host_path, Some("jet-host.js"));
        assert_eq!(m.build.rustc_target, "wasm32-unknown-unknown");
        assert_eq!(m.build.tsx_lowering, TSX_LOWERING_STRICT);
        assert!(m.source.jet_config_hash.starts_with("sha256:"));
    }

    #[test]
    fn desktop_manifest_inherits_web_profile_and_marks_tauri() {
        let tmp = TempDir::new().unwrap();
        let cfg = write_cfg(tmp.path(), "[wasm]\n");
        let m = Manifest::build(fixture_inputs(BuildTarget::Desktop, &cfg)).unwrap();
        assert_eq!(m.target, "desktop");
        assert_eq!(m.profile_target, "web");
        assert_eq!(m.package_for, Some("tauri"));
        assert_eq!(m.artifact.kind, "wasm");
    }

    #[test]
    fn tui_manifest_uses_native_artifact_and_no_wasm_paths() {
        let tmp = TempDir::new().unwrap();
        let cfg = write_cfg(tmp.path(), "[wasm]\n");
        let m = Manifest::build(fixture_inputs(BuildTarget::Tui, &cfg)).unwrap();
        assert_eq!(m.target, "tui");
        assert_eq!(m.profile_target, "tui");
        assert_eq!(m.package_for, Some("ratatui"));
        assert_eq!(m.artifact.kind, "native-bin");
        assert!(m.artifact.wasm_path.is_none());
        assert!(m.artifact.boot_path.is_none());
        assert!(m.artifact.host_path.is_none());
        assert!(m.artifact.html_path.is_none());
        assert_eq!(m.build.rustc_target, "host");
    }

    #[test]
    fn jet_config_hash_changes_with_contents() {
        let tmp = TempDir::new().unwrap();
        let cfg_a = write_cfg(tmp.path(), "[wasm]\nentry = 'a.tsx'\n");
        let m_a = Manifest::build(fixture_inputs(BuildTarget::Web, &cfg_a)).unwrap();
        fs::write(&cfg_a, "[wasm]\nentry = 'b.tsx'\n").unwrap();
        let m_b = Manifest::build(fixture_inputs(BuildTarget::Web, &cfg_a)).unwrap();
        assert_ne!(m_a.source.jet_config_hash, m_b.source.jet_config_hash);
    }

    #[test]
    fn pretty_json_is_stable_shape() {
        let tmp = TempDir::new().unwrap();
        let cfg = write_cfg(tmp.path(), "[wasm]\n");
        let m = Manifest::build(fixture_inputs(BuildTarget::Web, &cfg)).unwrap();
        let json = serde_json::to_value(&m).unwrap();
        // Spot-check the published schema fields exist.
        for key in [
            "schema_version",
            "target",
            "profile_target",
            "artifact",
            "build",
            "source",
        ] {
            assert!(json.get(key).is_some(), "missing top-level key {key}");
        }
        for key in ["mode", "rustc_target", "cargo_features", "tsx_lowering"] {
            assert!(json.pointer(&format!("/build/{key}")).is_some());
        }
        for key in ["entry", "root_component", "jet_config_hash"] {
            assert!(json.pointer(&format!("/source/{key}")).is_some());
        }
    }
}
// CODEGEN-END
