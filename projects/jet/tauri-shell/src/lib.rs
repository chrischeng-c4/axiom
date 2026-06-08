// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
// CODEGEN-BEGIN
//! `jet-tauri-shell` — desktop packaging for Jet apps via Tauri.
//!
//! @spec `.score/tech_design/projects/jet/logic/multi-target/desktop-runtime.md`
//! @issue #1242
//!
//! ## Slice 2 scope (this commit)
//!
//! Public surface only — no hard `tauri` crate dependency yet:
//!
//! - [`BundleManifest`] mirrors the JSON shape that
//!   `projects/jet/src/wasm_build/manifest.rs` emits to
//!   `dist/jet-target.json`. Adds a strict `validate()` that
//!   refuses non-desktop / non-tauri / wrong-schema-version
//!   bundles loud-fast.
//! - [`WindowConfig`] is the OS-window config the shell will hand
//!   to the future `tauri::Builder`. Defaults match the spec
//!   (1280x800, resizable, "Jet App").
//! - [`bridge::BackendBridge`] is the transport-agnostic IPC
//!   trait the substrate exposes. Slice 2b wires it through
//!   `tauri::ipc::Invoke`; Slice 5 wires the Cue concrete bridge.
//!
//! Real Tauri integration (Builder + window construction + the
//! `tauri::command` IPC adapter) lands in Slice 2b once we pick a
//! tauri minor version and add it as a feature-gated dependency.
//! Slice 4 will then package the WASM bundle into a real desktop
//! app via `tauri build`.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub mod bridge;
pub mod lifecycle;
pub mod packager;

/// Schema version this crate understands. Must match
/// [`jet::wasm_build::manifest::SCHEMA_VERSION`] (currently `1`).
pub const SUPPORTED_SCHEMA_VERSION: u32 = 1;

/// Deserialized `dist/jet-target.json`. The shape mirrors the
/// emitter at `projects/jet/src/wasm_build/manifest.rs` exactly. We
/// duplicate the struct here (rather than depending on that crate)
/// because the desktop packager is a downstream consumer — it
/// MUST be able to parse manifests the user could have produced
/// with an older or vendored version of `jet`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct BundleManifest {
    pub schema_version: u32,
    pub target: String,
    pub profile_target: String,
    /// `Some("tauri")` for desktop bundles. `None` for web.
    /// `Some("ratatui")` for TUI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_for: Option<String>,
    pub artifact: Artifact,
    pub build: Build,
    pub source: Source,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Artifact {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wasm_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boot_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_path: Option<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Build {
    pub mode: String,
    pub rustc_target: String,
    pub cargo_features: Vec<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Source {
    pub entry: String,
    pub root_component: String,
    pub jet_config_hash: String,
}

/// Errors the manifest pipeline emits before `tauri::Builder` is
/// even invoked. All variants are unrecoverable from inside the
/// packager — surface them to the user.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("manifest file missing at {0}")]
    NotFound(PathBuf),
    #[error("reading manifest at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("parsing manifest at {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error(
        "manifest schema_version {actual} not supported (expected {SUPPORTED_SCHEMA_VERSION})"
    )]
    UnsupportedSchemaVersion { actual: u32 },
    #[error("manifest target {actual:?}, expected \"desktop\" — refusing to package")]
    WrongTarget { actual: String },
    #[error("manifest package_for {actual:?}, expected Some(\"tauri\") — refusing to package")]
    WrongPackageFor { actual: Option<String> },
    #[error(
        "artifact.kind {actual:?}, expected \"wasm\" — desktop packager needs the WASM bundle"
    )]
    WrongArtifactKind { actual: String },
    #[error("artifact.html_path missing — Tauri shell needs an entry HTML to load")]
    MissingHtmlPath,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
impl BundleManifest {
    /// Read and parse `<dir>/jet-target.json`. Does NOT validate
    /// — call [`validate_for_desktop`](Self::validate_for_desktop)
    /// next.
    pub fn from_artifact_dir(dir: &Path) -> Result<Self, ManifestError> {
        let path = dir.join("jet-target.json");
        Self::from_path(&path)
    }

    /// Read and parse a manifest at the exact path given. Used by
    /// tests and by callers that don't follow the conventional
    /// `dist/<target>/jet-target.json` layout.
    pub fn from_path(path: &Path) -> Result<Self, ManifestError> {
        if !path.exists() {
            return Err(ManifestError::NotFound(path.to_path_buf()));
        }
        let bytes = fs::read(path).map_err(|source| ManifestError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let manifest: BundleManifest =
            serde_json::from_slice(&bytes).map_err(|source| ManifestError::Parse {
                path: path.to_path_buf(),
                source,
            })?;
        Ok(manifest)
    }

    /// Strict validation: the manifest must describe a desktop
    /// bundle the Tauri shell can package.
    pub fn validate_for_desktop(&self) -> Result<(), ManifestError> {
        if self.schema_version != SUPPORTED_SCHEMA_VERSION {
            return Err(ManifestError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }
        if self.target != "desktop" {
            return Err(ManifestError::WrongTarget {
                actual: self.target.clone(),
            });
        }
        if self.package_for.as_deref() != Some("tauri") {
            return Err(ManifestError::WrongPackageFor {
                actual: self.package_for.clone(),
            });
        }
        if self.artifact.kind != "wasm" {
            return Err(ManifestError::WrongArtifactKind {
                actual: self.artifact.kind.clone(),
            });
        }
        if self.artifact.html_path.is_none() {
            return Err(ManifestError::MissingHtmlPath);
        }
        Ok(())
    }
}

/// OS-window config for the Tauri shell. Defaults match the
/// values pinned in `desktop-runtime.md` §"Public surface".
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Jet App".into(),
            width: 1280,
            height: 800,
            resizable: true,
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
impl WindowConfig {
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    pub fn locked(mut self) -> Self {
        self.resizable = false;
        self
    }
}

/// The desktop shell's launch handle. Slice 2 stops at "validated
/// inputs"; Slice 2b plugs in `tauri::Builder` behind a feature
/// flag. The public surface is locked here so downstream callers
/// (Cue's main, conformance harness) can compile against the
/// real public API today.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Clone)]
pub struct TauriShell {
    manifest: BundleManifest,
    artifact_dir: PathBuf,
    window: WindowConfig,
    lifecycle: lifecycle::LifecycleBus,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
impl TauriShell {
    /// Construct from an artifact directory containing
    /// `jet-target.json`. Runs full validation eagerly.
    pub fn from_artifact_dir(artifact_dir: &Path) -> Result<Self, ManifestError> {
        let manifest = BundleManifest::from_artifact_dir(artifact_dir)?;
        manifest.validate_for_desktop()?;
        Ok(Self {
            manifest,
            artifact_dir: artifact_dir.to_path_buf(),
            window: WindowConfig::default(),
            lifecycle: lifecycle::LifecycleBus::new(),
        })
    }

    pub fn with_window(mut self, window: WindowConfig) -> Self {
        self.window = window;
        self
    }

    /// Replace the in-process [`lifecycle::LifecycleBus`] with a
    /// caller-supplied one. Useful when the substrate wants to
    /// share a single bus across multiple shells (e.g. a
    /// multi-window Cue session) or pre-register listeners
    /// before the shell launches.
    pub fn with_lifecycle_bus(mut self, bus: lifecycle::LifecycleBus) -> Self {
        self.lifecycle = bus;
        self
    }

    /// Borrow the lifecycle bus so callers can `subscribe(...)` /
    /// `publish(...)` without moving the shell. The returned
    /// reference is `Clone` (the bus is `Arc<Mutex<_>>`-shared);
    /// callers can clone it for cross-thread use.
    pub fn lifecycle(&self) -> &lifecycle::LifecycleBus {
        &self.lifecycle
    }

    pub fn manifest(&self) -> &BundleManifest {
        &self.manifest
    }

    pub fn artifact_dir(&self) -> &Path {
        &self.artifact_dir
    }

    pub fn window(&self) -> &WindowConfig {
        &self.window
    }

    /// Compute the [`packager::PackagePlan`] for this shell into
    /// the given output root (`<output_root>/desktop/<os>/` and
    /// `<output_root>/tauri-src/dist/` are the resulting trees).
    /// Pure function; Slice 4b's executor consumes the plan.
    pub fn plan_package(
        &self,
        output_root: &Path,
    ) -> Result<packager::PackagePlan, packager::PackagePlanError> {
        packager::plan_package(self, output_root)
    }

    /// Resolve the entry HTML path on disk (`artifact_dir` joined
    /// with `manifest.artifact.html_path`). The Tauri shell will
    /// hand this URI to the webview in Slice 2b.
    pub fn entry_html_path(&self) -> PathBuf {
        let html = self
            .manifest
            .artifact
            .html_path
            .as_deref()
            .expect("validate_for_desktop guarantees html_path is Some");
        self.artifact_dir.join(html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_manifest(dir: &Path, body: &str) -> PathBuf {
        let p = dir.join("jet-target.json");
        let mut f = fs::File::create(&p).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        p
    }

    fn desktop_manifest_json() -> &'static str {
        r#"{
            "schema_version": 1,
            "target": "desktop",
            "profile_target": "web",
            "package_for": "tauri",
            "artifact": {
                "kind": "wasm",
                "wasm_path": "app_bg.wasm",
                "boot_path": "boot.js",
                "host_path": "jet-host.js",
                "html_path": "index.html"
            },
            "build": {
                "mode": "release",
                "rustc_target": "wasm32-unknown-unknown",
                "cargo_features": [
                    "jet-multi-target/target-web",
                    "jet-multi-target/target-desktop"
                ]
            },
            "source": {
                "entry": "src/index.tsx",
                "root_component": "App",
                "jet_config_hash": "sha256:deadbeef"
            }
        }"#
    }

    #[test]
    fn parses_full_desktop_manifest() {
        let tmp = TempDir::new().unwrap();
        let path = write_manifest(tmp.path(), desktop_manifest_json());
        let m = BundleManifest::from_path(&path).unwrap();
        assert_eq!(m.schema_version, 1);
        assert_eq!(m.target, "desktop");
        assert_eq!(m.profile_target, "web");
        assert_eq!(m.package_for.as_deref(), Some("tauri"));
        assert_eq!(m.artifact.kind, "wasm");
        assert_eq!(m.artifact.html_path.as_deref(), Some("index.html"));
        assert!(m.build.cargo_features.iter().any(|f| f.contains("desktop")));
    }

    #[test]
    fn from_artifact_dir_reads_jet_target_json_in_dir() {
        let tmp = TempDir::new().unwrap();
        write_manifest(tmp.path(), desktop_manifest_json());
        let m = BundleManifest::from_artifact_dir(tmp.path()).unwrap();
        assert_eq!(m.target, "desktop");
    }

    #[test]
    fn missing_manifest_returns_not_found_with_path() {
        let tmp = TempDir::new().unwrap();
        let err = BundleManifest::from_artifact_dir(tmp.path()).unwrap_err();
        match err {
            ManifestError::NotFound(p) => {
                assert!(p.ends_with("jet-target.json"));
            }
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[test]
    fn malformed_json_returns_parse_error() {
        let tmp = TempDir::new().unwrap();
        let p = write_manifest(tmp.path(), "{ not valid json");
        let err = BundleManifest::from_path(&p).unwrap_err();
        assert!(matches!(err, ManifestError::Parse { .. }));
    }

    #[test]
    fn validate_passes_on_desktop_tauri_wasm_v1() {
        let tmp = TempDir::new().unwrap();
        let p = write_manifest(tmp.path(), desktop_manifest_json());
        let m = BundleManifest::from_path(&p).unwrap();
        m.validate_for_desktop().unwrap();
    }

    #[test]
    fn validate_rejects_unknown_schema_version() {
        let json =
            desktop_manifest_json().replace("\"schema_version\": 1", "\"schema_version\": 99");
        let tmp = TempDir::new().unwrap();
        let p = write_manifest(tmp.path(), &json);
        let m = BundleManifest::from_path(&p).unwrap();
        match m.validate_for_desktop().unwrap_err() {
            ManifestError::UnsupportedSchemaVersion { actual } => assert_eq!(actual, 99),
            other => panic!("expected UnsupportedSchemaVersion, got {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_tui_target() {
        let json = desktop_manifest_json()
            .replace("\"target\": \"desktop\"", "\"target\": \"tui\"")
            .replace("\"package_for\": \"tauri\"", "\"package_for\": \"ratatui\"");
        let tmp = TempDir::new().unwrap();
        let p = write_manifest(tmp.path(), &json);
        let m = BundleManifest::from_path(&p).unwrap();
        match m.validate_for_desktop().unwrap_err() {
            ManifestError::WrongTarget { actual } => assert_eq!(actual, "tui"),
            other => panic!("expected WrongTarget, got {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_web_bundle_with_no_package_for() {
        // Web bundles emit `target: "desktop"` only when
        // `--target desktop` is used; a plain `--target web`
        // produces target=web which `WrongTarget` catches first.
        // This case asserts the package_for guard still applies
        // when someone hand-edits the manifest.
        let json = desktop_manifest_json().replace(",\n            \"package_for\": \"tauri\"", "");
        let tmp = TempDir::new().unwrap();
        let p = write_manifest(tmp.path(), &json);
        let m = BundleManifest::from_path(&p).unwrap();
        match m.validate_for_desktop().unwrap_err() {
            ManifestError::WrongPackageFor { actual } => assert_eq!(actual, None),
            other => panic!("expected WrongPackageFor, got {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_native_artifact_kind() {
        let json =
            desktop_manifest_json().replace("\"kind\": \"wasm\"", "\"kind\": \"native-bin\"");
        let tmp = TempDir::new().unwrap();
        let p = write_manifest(tmp.path(), &json);
        let m = BundleManifest::from_path(&p).unwrap();
        match m.validate_for_desktop().unwrap_err() {
            ManifestError::WrongArtifactKind { actual } => assert_eq!(actual, "native-bin"),
            other => panic!("expected WrongArtifactKind, got {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_manifest_missing_html_path() {
        let json =
            desktop_manifest_json().replace(",\n                \"html_path\": \"index.html\"", "");
        let tmp = TempDir::new().unwrap();
        let p = write_manifest(tmp.path(), &json);
        let m = BundleManifest::from_path(&p).unwrap();
        match m.validate_for_desktop().unwrap_err() {
            ManifestError::MissingHtmlPath => {}
            other => panic!("expected MissingHtmlPath, got {other:?}"),
        }
    }

    #[test]
    fn window_config_defaults_match_spec() {
        let w = WindowConfig::default();
        assert_eq!(w.title, "Jet App");
        assert_eq!(w.width, 1280);
        assert_eq!(w.height, 800);
        assert!(w.resizable);
    }

    #[test]
    fn window_config_builders_compose() {
        let w = WindowConfig::default()
            .with_title("Cue")
            .with_size(1024, 768)
            .locked();
        assert_eq!(w.title, "Cue");
        assert_eq!(w.width, 1024);
        assert_eq!(w.height, 768);
        assert!(!w.resizable);
    }

    #[test]
    fn shell_construct_validates_eagerly() {
        let tmp = TempDir::new().unwrap();
        write_manifest(tmp.path(), desktop_manifest_json());
        let shell = TauriShell::from_artifact_dir(tmp.path()).unwrap();
        assert_eq!(shell.manifest().target, "desktop");
        assert_eq!(shell.window().title, "Jet App");
        assert!(shell.entry_html_path().ends_with("index.html"));
    }

    #[test]
    fn shell_with_window_overrides_default() {
        let tmp = TempDir::new().unwrap();
        write_manifest(tmp.path(), desktop_manifest_json());
        let shell = TauriShell::from_artifact_dir(tmp.path())
            .unwrap()
            .with_window(WindowConfig::default().with_title("Cue Desktop"));
        assert_eq!(shell.window().title, "Cue Desktop");
    }

    #[test]
    fn shell_plan_package_returns_three_copies_for_valid_manifest() {
        let tmp = TempDir::new().unwrap();
        write_manifest(tmp.path(), desktop_manifest_json());
        let shell = TauriShell::from_artifact_dir(tmp.path()).unwrap();
        let plan = shell.plan_package(std::path::Path::new("/out")).unwrap();
        assert_eq!(plan.copies.len(), 3);
        assert_eq!(plan.command.program, "tauri");
        assert!(plan.output_dir.starts_with("/out/desktop/"));
    }

    #[test]
    fn shell_lifecycle_bus_starts_with_zero_listeners() {
        let tmp = TempDir::new().unwrap();
        write_manifest(tmp.path(), desktop_manifest_json());
        let shell = TauriShell::from_artifact_dir(tmp.path()).unwrap();
        assert_eq!(shell.lifecycle().listener_count(), 0);
    }

    #[test]
    fn shell_with_lifecycle_bus_swaps_pre_subscribed_bus() {
        use lifecycle::{LifecycleBus, LifecycleEvent, LifecycleListener, WindowId};
        use std::sync::Mutex;

        struct Counter(Mutex<u32>);
        impl LifecycleListener for std::sync::Arc<Counter> {
            fn on_event(&self, _e: &LifecycleEvent) {
                *self.0.lock().unwrap() += 1;
            }
        }

        let tmp = TempDir::new().unwrap();
        write_manifest(tmp.path(), desktop_manifest_json());

        let bus = LifecycleBus::new();
        let counter = std::sync::Arc::new(Counter(Mutex::new(0)));
        bus.subscribe(counter.clone());

        let shell = TauriShell::from_artifact_dir(tmp.path())
            .unwrap()
            .with_lifecycle_bus(bus);
        assert_eq!(shell.lifecycle().listener_count(), 1);

        shell.lifecycle().publish(LifecycleEvent::WindowCreated {
            window: WindowId::main(),
        });
        assert_eq!(*counter.0.lock().unwrap(), 1);
    }

    #[test]
    fn shell_construct_propagates_validation_error() {
        let json =
            desktop_manifest_json().replace("\"target\": \"desktop\"", "\"target\": \"web\"");
        let tmp = TempDir::new().unwrap();
        write_manifest(tmp.path(), &json);
        let err = TauriShell::from_artifact_dir(tmp.path()).unwrap_err();
        assert!(matches!(err, ManifestError::WrongTarget { .. }));
    }
}
// CODEGEN-END
