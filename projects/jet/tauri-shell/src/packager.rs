// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
// CODEGEN-BEGIN
//! Packager for the desktop target — turns a validated
//! `dist/jet-target.json` artifact into a [`PackagePlan`] that
//! describes the file copies + `tauri build` invocation Slice 4b
//! will execute.
//!
//! @spec `.score/tech_design/projects/jet/logic/multi-target/desktop-runtime.md`
//!     §"Slice 4 — packager + `jet-target.json` consumer".
//! @issue #1242 — Slice 4a (this commit): planning logic only, no
//!     filesystem mutation, no `tauri build` exec. The plan is a
//!     pure function of the [`TauriShell`] state + an output
//!     directory, so tests can assert exact copies + exact command
//!     line without spinning up a webview or shelling out.
//!
//! Why the split.  Bundling planning in front of execution lets
//! the dry-run path (`jet build --target desktop --package
//! --dry-run`, planned for Slice 4b) print the plan instead of
//! running it; lets CI assert plan stability under spec drift;
//! and decouples the per-OS output-tree shape from the actual
//! `tauri build` arguments.

use crate::{BundleManifest, TauriShell, WindowConfig};
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

/// A single file the packager will copy from the WASM artifact
/// directory into the Tauri source tree before invoking
/// `tauri build`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileCopy {
    pub source: PathBuf,
    pub dest: PathBuf,
}

/// The shell command Slice 4b will exec to actually package the
/// app. Kept structural (program + args + working_dir) so unit
/// tests can assert the exact invocation without process
/// orchestration.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedCommand {
    pub program: String,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
}

/// Host operating system the plan was computed for. Drives the
/// `dist/desktop/{macos,linux,windows}/` subdirectory selection.
/// We default to the build host, but tests + cross-builds can
/// override via [`PackagePlan::for_host`].
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostOs {
    Macos,
    Linux,
    Windows,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
impl HostOs {
    /// The current build host. Falls back to `Linux` for any OS
    /// not in the `{macos, linux, windows}` triplet — Slice 4b
    /// will refuse to package on those, but the plan itself is
    /// still computable so tests stay portable.
    pub fn current() -> Self {
        match std::env::consts::OS {
            "macos" => HostOs::Macos,
            "windows" => HostOs::Windows,
            _ => HostOs::Linux,
        }
    }

    /// Subdirectory name under `dist/desktop/` per the spec.
    pub fn dist_subdir(&self) -> &'static str {
        match self {
            HostOs::Macos => "macos",
            HostOs::Linux => "linux",
            HostOs::Windows => "windows",
        }
    }
}

/// Errors the planner emits when the inputs are well-formed at
/// the manifest layer but unfit for packaging (missing artifact
/// fields, output directory above the workspace root, etc.).
/// Distinct from [`crate::ManifestError`] which guards the
/// upstream parse + validate pass.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PackagePlanError {
    #[error("manifest artifact.html_path missing — packager needs an entry HTML")]
    MissingHtmlPath,
    #[error("manifest artifact.wasm_path missing — packager needs the WASM binary")]
    MissingWasmPath,
    #[error("manifest artifact.boot_path missing — packager needs the boot loader")]
    MissingBootPath,
}

/// Everything the Slice 4b executor needs to package the desktop
/// app. Returned by [`plan_package`] and exposed on
/// [`TauriShell::plan_package`].
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagePlan {
    /// WASM artifact directory the bundle lives in. The planner
    /// reads relative paths out of the manifest and joins them
    /// against this root.
    pub source_artifact_dir: PathBuf,
    /// Where the planner will copy the WASM bundle into so
    /// Tauri's bundler can pick it up. Conventionally
    /// `<output_dir>/tauri-src/dist/`.
    pub tauri_src_dist: PathBuf,
    /// Final per-OS output tree the packaged app lands in
    /// (`<output_dir>/desktop/<os>/`).
    pub output_dir: PathBuf,
    /// Files the planner will copy. Order is stable (HTML first,
    /// then WASM, then boot loader, then optional host adapter) so
    /// `assert_eq!` on a plan is meaningful in tests.
    pub copies: Vec<FileCopy>,
    /// The `tauri build` invocation Slice 4b will exec.
    pub command: PlannedCommand,
    /// Carried through so executors and dry-run formatters can
    /// surface the OS-window config without re-reading the
    /// shell.
    pub window: WindowConfig,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
impl PackagePlan {
    /// Produce a plan for an explicit host OS. Useful for tests
    /// (assert exact paths regardless of where CI runs) and for
    /// the future cross-compile path.
    pub fn for_host(
        manifest: &BundleManifest,
        artifact_dir: &Path,
        output_root: &Path,
        host: HostOs,
        window: WindowConfig,
    ) -> Result<Self, PackagePlanError> {
        let html_rel = manifest
            .artifact
            .html_path
            .as_deref()
            .ok_or(PackagePlanError::MissingHtmlPath)?;
        let wasm_rel = manifest
            .artifact
            .wasm_path
            .as_deref()
            .ok_or(PackagePlanError::MissingWasmPath)?;
        let boot_rel = manifest
            .artifact
            .boot_path
            .as_deref()
            .ok_or(PackagePlanError::MissingBootPath)?;

        let tauri_src = output_root.join("tauri-src");
        let tauri_src_dist = tauri_src.join("dist");
        let output_dir = output_root.join("desktop").join(host.dist_subdir());

        let mut copies = vec![
            FileCopy {
                source: artifact_dir.join(html_rel),
                dest: tauri_src_dist.join(html_rel),
            },
            FileCopy {
                source: artifact_dir.join(wasm_rel),
                dest: tauri_src_dist.join(wasm_rel),
            },
            FileCopy {
                source: artifact_dir.join(boot_rel),
                dest: tauri_src_dist.join(boot_rel),
            },
        ];
        if let Some(host_rel) = manifest.artifact.host_path.as_deref() {
            copies.push(FileCopy {
                source: artifact_dir.join(host_rel),
                dest: tauri_src_dist.join(host_rel),
            });
        }

        let command = PlannedCommand {
            program: "tauri".into(),
            args: vec!["build".into()],
            working_dir: tauri_src.clone(),
        };

        Ok(Self {
            source_artifact_dir: artifact_dir.to_path_buf(),
            tauri_src_dist,
            output_dir,
            copies,
            command,
            window,
        })
    }

    /// Convenience shorthand — uses the build host's OS.
    pub fn for_current_host(
        manifest: &BundleManifest,
        artifact_dir: &Path,
        output_root: &Path,
        window: WindowConfig,
    ) -> Result<Self, PackagePlanError> {
        Self::for_host(
            manifest,
            artifact_dir,
            output_root,
            HostOs::current(),
            window,
        )
    }

    /// Files this plan will read from. Slice 4b's executor calls
    /// this to pre-flight existence checks before mutating
    /// anything.
    pub fn source_paths(&self) -> Vec<&Path> {
        self.copies.iter().map(|c| c.source.as_path()).collect()
    }

    /// Files this plan will write to. Slice 4b uses this to
    /// `mkdir -p` the parent dirs once per unique parent.
    pub fn dest_paths(&self) -> Vec<&Path> {
        self.copies.iter().map(|c| c.dest.as_path()).collect()
    }
}

/// Public entry point — see [`TauriShell::plan_package`].
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
pub fn plan_package(
    shell: &TauriShell,
    output_root: &Path,
) -> Result<PackagePlan, PackagePlanError> {
    PackagePlan::for_current_host(
        shell.manifest(),
        shell.artifact_dir(),
        output_root,
        shell.window().clone(),
    )
}

/// Errors the [`execute_copies`] file-copy stage can surface.
/// Distinct from [`PackagePlanError`] (which guards plan
/// computation) — any of these come from real filesystem
/// interactions and are recoverable only by the caller fixing
/// inputs / permissions.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("source artifact missing at {0}")]
    SourceMissing(PathBuf),
    #[error("creating directory {path}: {source}")]
    MkDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("copying {source_path} → {dest_path}: {source}")]
    Copy {
        source_path: PathBuf,
        dest_path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("packager command spawn failed for {program:?}: {source}")]
    ExecSpawn {
        program: String,
        #[source]
        source: std::io::Error,
    },
    #[error("packager command {program:?} exited with {status}")]
    ExecFailed { program: String, status: ExitStatus },
}

/// Summary of one [`execute_copies`] run. Returned to the caller
/// (and printed by the future `--package --dry-run` formatter)
/// so CI logs can show "copied N files / M bytes" without
/// re-walking the destination tree.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CopyReport {
    pub files_copied: usize,
    pub total_bytes: u64,
    pub dirs_created: usize,
}

/// Execute the file-copy stage of a [`PackagePlan`]:
///   1. Pre-flight: every `copy.source` must exist (loud-fast on
///      missing — we don't want a partial copy).
///   2. `mkdir -p` each unique destination parent (deduplicated).
///   3. `fs::copy` each `source` → `dest` in plan order.
///
/// Pure file mutation — does NOT invoke `tauri build`. The Slice
/// 4c executor wraps this with the actual command exec once a
/// tauri version is pinned (Slice 2b).
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
pub fn execute_copies(plan: &PackagePlan) -> Result<CopyReport, PackageError> {
    for copy in &plan.copies {
        if !copy.source.exists() {
            return Err(PackageError::SourceMissing(copy.source.clone()));
        }
    }

    let mut dirs_created = 0usize;
    let mut seen_dirs: Vec<PathBuf> = Vec::new();
    for copy in &plan.copies {
        if let Some(parent) = copy.dest.parent() {
            if seen_dirs.iter().any(|d| d == parent) {
                continue;
            }
            seen_dirs.push(parent.to_path_buf());
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|source| PackageError::MkDir {
                    path: parent.to_path_buf(),
                    source,
                })?;
                dirs_created += 1;
            }
        }
    }

    let mut total_bytes = 0u64;
    for copy in &plan.copies {
        let bytes =
            std::fs::copy(&copy.source, &copy.dest).map_err(|source| PackageError::Copy {
                source_path: copy.source.clone(),
                dest_path: copy.dest.clone(),
                source,
            })?;
        total_bytes += bytes;
    }

    Ok(CopyReport {
        files_copied: plan.copies.len(),
        total_bytes,
        dirs_created,
    })
}

/// Summary of one [`execute_command`] run. Returned to CI logs and
/// the future `--package` formatter so the actual invocation that
/// shipped is visible without re-walking the plan.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandReport {
    pub program: String,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
}

/// Execute the `command` half of a [`PackagePlan`]: spawn
/// `command.program command.args` from `command.working_dir` and
/// wait for it. Pure shell-out; the spec gates the actual `tauri
/// build` semantics on a pinned tauri minor version (Slice 2b),
/// but the executor itself is version-agnostic — it runs whatever
/// [`PackagePlan::command`] resolves to, so callers can stub the
/// program (e.g. `echo`) in tests.
///
/// Surfaces three failure modes:
/// - [`PackageError::ExecSpawn`] — the OS refused to spawn the
///   program (typically "binary not on PATH").
/// - [`PackageError::ExecFailed`] — the program ran but exited
///   non-zero. Carries the [`ExitStatus`] verbatim so callers can
///   inspect the code / signal.
/// - any of the existing copy-stage variants are *not* re-emitted
///   here — [`execute_copies`] is the upstream gate.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
pub fn execute_command(plan: &PackagePlan) -> Result<CommandReport, PackageError> {
    let cmd = &plan.command;
    let status = std::process::Command::new(&cmd.program)
        .args(&cmd.args)
        .current_dir(&cmd.working_dir)
        .status()
        .map_err(|source| PackageError::ExecSpawn {
            program: cmd.program.clone(),
            source,
        })?;
    if !status.success() {
        return Err(PackageError::ExecFailed {
            program: cmd.program.clone(),
            status,
        });
    }
    Ok(CommandReport {
        program: cmd.program.clone(),
        args: cmd.args.clone(),
        working_dir: cmd.working_dir.clone(),
    })
}

/// Render a [`PackagePlan`] as a human-readable transcript for
/// `jet build --target desktop --package --dry-run` (the CLI
/// wire-up itself lands in a follow-up sub-slice). Stable line
/// order so byte-equality assertions are meaningful in tests.
///
/// Layout:
///
/// ```text
/// PackagePlan {
///   source_artifact_dir: <path>
///   tauri_src_dist:      <path>
///   output_dir:          <path>
///   window: <title> @ <w>x<h> (resizable: <bool>)
///   copies (<n>):
///     <i>. <source> -> <dest>
///   command: <program> [<args...>] (cwd: <working_dir>)
/// }
/// ```
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
pub fn format_dry_run(plan: &PackagePlan) -> String {
    let mut out = String::new();
    out.push_str("PackagePlan {\n");
    out.push_str(&format!(
        "  source_artifact_dir: {}\n",
        plan.source_artifact_dir.display()
    ));
    out.push_str(&format!(
        "  tauri_src_dist:      {}\n",
        plan.tauri_src_dist.display()
    ));
    out.push_str(&format!(
        "  output_dir:          {}\n",
        plan.output_dir.display()
    ));
    out.push_str(&format!(
        "  window: {} @ {}x{} (resizable: {})\n",
        plan.window.title, plan.window.width, plan.window.height, plan.window.resizable
    ));
    out.push_str(&format!("  copies ({}):\n", plan.copies.len()));
    for (i, copy) in plan.copies.iter().enumerate() {
        out.push_str(&format!(
            "    {}. {} -> {}\n",
            i + 1,
            copy.source.display(),
            copy.dest.display()
        ));
    }
    let args_joined = if plan.command.args.is_empty() {
        String::new()
    } else {
        format!(" {}", plan.command.args.join(" "))
    };
    out.push_str(&format!(
        "  command: {}{} (cwd: {})\n",
        plan.command.program,
        args_joined,
        plan.command.working_dir.display()
    ));
    out.push_str("}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Artifact, Build, Source};

    fn manifest_with(artifact: Artifact) -> BundleManifest {
        BundleManifest {
            schema_version: 1,
            target: "desktop".into(),
            profile_target: "web".into(),
            package_for: Some("tauri".into()),
            artifact,
            build: Build {
                mode: "release".into(),
                rustc_target: "wasm32-unknown-unknown".into(),
                cargo_features: vec![
                    "jet-multi-target/target-web".into(),
                    "jet-multi-target/target-desktop".into(),
                ],
            },
            source: Source {
                entry: "src/index.tsx".into(),
                root_component: "App".into(),
                jet_config_hash: "sha256:abc".into(),
            },
        }
    }

    fn good_artifact() -> Artifact {
        Artifact {
            kind: "wasm".into(),
            wasm_path: Some("app_bg.wasm".into()),
            boot_path: Some("boot.js".into()),
            host_path: Some("jet-host.js".into()),
            html_path: Some("index.html".into()),
        }
    }

    #[test]
    fn host_os_subdirs_match_spec() {
        assert_eq!(HostOs::Macos.dist_subdir(), "macos");
        assert_eq!(HostOs::Linux.dist_subdir(), "linux");
        assert_eq!(HostOs::Windows.dist_subdir(), "windows");
    }

    #[test]
    fn for_host_stages_expected_wasm_file_copies_in_order() {
        let m = manifest_with(good_artifact());
        let plan = PackagePlan::for_host(
            &m,
            Path::new("/in"),
            Path::new("/out"),
            HostOs::Macos,
            WindowConfig::default(),
        )
        .unwrap();
        assert_eq!(plan.copies.len(), 4);
        assert_eq!(plan.copies[0].source, PathBuf::from("/in/index.html"));
        assert_eq!(
            plan.copies[0].dest,
            PathBuf::from("/out/tauri-src/dist/index.html")
        );
        assert_eq!(plan.copies[1].source, PathBuf::from("/in/app_bg.wasm"));
        assert_eq!(plan.copies[2].source, PathBuf::from("/in/boot.js"));
        assert_eq!(plan.copies[3].source, PathBuf::from("/in/jet-host.js"));
    }

    #[test]
    fn for_host_keeps_legacy_three_file_manifest_compatible() {
        let m = manifest_with(Artifact {
            host_path: None,
            ..good_artifact()
        });
        let plan = PackagePlan::for_host(
            &m,
            Path::new("/in"),
            Path::new("/out"),
            HostOs::Macos,
            WindowConfig::default(),
        )
        .unwrap();
        assert_eq!(plan.copies.len(), 3);
        assert_eq!(plan.copies[2].source, PathBuf::from("/in/boot.js"));
    }

    #[test]
    fn for_host_emits_per_os_output_dir() {
        let m = manifest_with(good_artifact());
        for (host, leaf) in [
            (HostOs::Macos, "macos"),
            (HostOs::Linux, "linux"),
            (HostOs::Windows, "windows"),
        ] {
            let plan = PackagePlan::for_host(
                &m,
                Path::new("/in"),
                Path::new("/out"),
                host,
                WindowConfig::default(),
            )
            .unwrap();
            assert_eq!(
                plan.output_dir,
                PathBuf::from(format!("/out/desktop/{leaf}"))
            );
        }
    }

    #[test]
    fn for_host_planned_command_runs_tauri_build_in_src() {
        let m = manifest_with(good_artifact());
        let plan = PackagePlan::for_host(
            &m,
            Path::new("/in"),
            Path::new("/out"),
            HostOs::Linux,
            WindowConfig::default(),
        )
        .unwrap();
        assert_eq!(plan.command.program, "tauri");
        assert_eq!(plan.command.args, vec!["build".to_string()]);
        assert_eq!(plan.command.working_dir, PathBuf::from("/out/tauri-src"));
    }

    #[test]
    fn for_host_carries_window_config_through() {
        let m = manifest_with(good_artifact());
        let win = WindowConfig::default()
            .with_title("Cue Desktop")
            .with_size(1024, 768)
            .locked();
        let plan = PackagePlan::for_host(
            &m,
            Path::new("/in"),
            Path::new("/out"),
            HostOs::Macos,
            win.clone(),
        )
        .unwrap();
        assert_eq!(plan.window, win);
    }

    #[test]
    fn missing_html_path_is_a_planning_error() {
        let m = manifest_with(Artifact {
            html_path: None,
            ..good_artifact()
        });
        match PackagePlan::for_host(
            &m,
            Path::new("/in"),
            Path::new("/out"),
            HostOs::Macos,
            WindowConfig::default(),
        )
        .unwrap_err()
        {
            PackagePlanError::MissingHtmlPath => {}
            other => panic!("expected MissingHtmlPath, got {other:?}"),
        }
    }

    #[test]
    fn missing_wasm_path_is_a_planning_error() {
        let m = manifest_with(Artifact {
            wasm_path: None,
            ..good_artifact()
        });
        assert_eq!(
            PackagePlan::for_host(
                &m,
                Path::new("/in"),
                Path::new("/out"),
                HostOs::Macos,
                WindowConfig::default(),
            )
            .unwrap_err(),
            PackagePlanError::MissingWasmPath,
        );
    }

    #[test]
    fn missing_boot_path_is_a_planning_error() {
        let m = manifest_with(Artifact {
            boot_path: None,
            ..good_artifact()
        });
        assert_eq!(
            PackagePlan::for_host(
                &m,
                Path::new("/in"),
                Path::new("/out"),
                HostOs::Macos,
                WindowConfig::default(),
            )
            .unwrap_err(),
            PackagePlanError::MissingBootPath,
        );
    }

    #[test]
    fn source_and_dest_paths_walk_each_copy_in_order() {
        let m = manifest_with(good_artifact());
        let plan = PackagePlan::for_host(
            &m,
            Path::new("/in"),
            Path::new("/out"),
            HostOs::Macos,
            WindowConfig::default(),
        )
        .unwrap();
        let sources: Vec<&Path> = plan.source_paths();
        assert_eq!(sources.len(), 4);
        assert!(sources.iter().all(|p| p.starts_with("/in")));
        let dests: Vec<&Path> = plan.dest_paths();
        assert!(dests.iter().all(|p| p.starts_with("/out/tauri-src/dist")));
    }

    // ---- Slice 4b: execute_copies file-copy executor ----

    /// Materialize `index.html`, `app_bg.wasm`, `boot.js`, and
    /// `jet-host.js` in
    /// `dir` with deterministic byte payloads so tests can
    /// assert exact `total_bytes`.
    fn seed_artifact_dir(dir: &Path) {
        std::fs::write(dir.join("index.html"), b"<!doctype html>").unwrap();
        std::fs::write(dir.join("app_bg.wasm"), b"\0asm\x01\x00\x00\x00").unwrap();
        std::fs::write(dir.join("boot.js"), b"// boot").unwrap();
        std::fs::write(dir.join("jet-host.js"), b"// host").unwrap();
    }

    fn plan_into(in_dir: &Path, out_root: &Path) -> PackagePlan {
        let m = manifest_with(good_artifact());
        PackagePlan::for_host(&m, in_dir, out_root, HostOs::Linux, WindowConfig::default()).unwrap()
    }

    #[test]
    fn execute_copies_writes_wasm_files_with_correct_byte_count() {
        let in_dir = tempfile::tempdir().unwrap();
        let out_dir = tempfile::tempdir().unwrap();
        seed_artifact_dir(in_dir.path());
        let plan = plan_into(in_dir.path(), out_dir.path());

        let report = execute_copies(&plan).unwrap();
        assert_eq!(report.files_copied, 4);
        // 15 (html) + 8 (wasm magic) + 7 (boot) + 7 (host) = 37 bytes.
        assert_eq!(report.total_bytes, 37);
        assert_eq!(report.dirs_created, 1); // tauri-src/dist created once.

        for copy in &plan.copies {
            assert!(copy.dest.exists(), "missing {}", copy.dest.display());
        }
    }

    #[test]
    fn execute_copies_idempotent_when_dest_dir_already_exists() {
        let in_dir = tempfile::tempdir().unwrap();
        let out_dir = tempfile::tempdir().unwrap();
        seed_artifact_dir(in_dir.path());
        let plan = plan_into(in_dir.path(), out_dir.path());

        // First run creates the dir.
        execute_copies(&plan).unwrap();
        // Second run sees the dir already there → dirs_created = 0.
        let report = execute_copies(&plan).unwrap();
        assert_eq!(report.dirs_created, 0);
        assert_eq!(report.files_copied, 4);
    }

    #[test]
    fn execute_copies_loud_fast_when_source_missing() {
        let in_dir = tempfile::tempdir().unwrap();
        let out_dir = tempfile::tempdir().unwrap();
        // Only seed two of three artifacts.
        std::fs::write(in_dir.path().join("index.html"), b"x").unwrap();
        std::fs::write(in_dir.path().join("app_bg.wasm"), b"x").unwrap();
        // boot.js intentionally missing.
        let plan = plan_into(in_dir.path(), out_dir.path());

        match execute_copies(&plan).unwrap_err() {
            PackageError::SourceMissing(p) => {
                assert!(p.ends_with("boot.js"), "got {}", p.display());
            }
            other => panic!("expected SourceMissing, got {other:?}"),
        }

        // Pre-flight failure must NOT leave a partial copy in
        // the dest tree.
        for copy in &plan.copies {
            assert!(
                !copy.dest.exists(),
                "pre-flight failed but dest exists: {}",
                copy.dest.display(),
            );
        }
    }

    #[test]
    fn execute_copies_dedupes_parent_dir_creation() {
        // All three copies share the same parent (tauri-src/dist),
        // so dirs_created stays at 1 even though we walk three
        // copies. This guards against accidental
        // create_dir_all-per-copy regressions.
        let in_dir = tempfile::tempdir().unwrap();
        let out_dir = tempfile::tempdir().unwrap();
        seed_artifact_dir(in_dir.path());
        let plan = plan_into(in_dir.path(), out_dir.path());

        let report = execute_copies(&plan).unwrap();
        assert_eq!(report.dirs_created, 1);
    }

    #[test]
    fn copy_report_default_is_zero() {
        let r = CopyReport::default();
        assert_eq!(
            r,
            CopyReport {
                files_copied: 0,
                total_bytes: 0,
                dirs_created: 0,
            }
        );
    }

    // ---- Slice 4c: execute_command + format_dry_run ----

    fn echo_plan(working_dir: &Path) -> PackagePlan {
        // Hand-craft a plan with a portable program (`echo`) that
        // exists on every supported host. Plan body is otherwise
        // irrelevant for the executor — only `command` matters.
        PackagePlan {
            source_artifact_dir: PathBuf::from("/in"),
            tauri_src_dist: PathBuf::from("/out/tauri-src/dist"),
            output_dir: PathBuf::from("/out/desktop/macos"),
            copies: vec![],
            command: PlannedCommand {
                program: "echo".into(),
                args: vec!["jet-packager-test".into()],
                working_dir: working_dir.to_path_buf(),
            },
            window: WindowConfig::default(),
        }
    }

    #[test]
    fn execute_command_returns_report_on_success() {
        // `echo` exits 0 on every supported host; the working_dir
        // must exist or `Command::status` errors before exec on
        // some platforms, so we point at a tempdir.
        let cwd = tempfile::tempdir().unwrap();
        let plan = echo_plan(cwd.path());
        let report = execute_command(&plan).unwrap();
        assert_eq!(report.program, "echo");
        assert_eq!(report.args, vec!["jet-packager-test".to_string()]);
        assert_eq!(report.working_dir, cwd.path());
    }

    #[test]
    fn execute_command_spawn_error_on_missing_program() {
        let cwd = tempfile::tempdir().unwrap();
        let mut plan = echo_plan(cwd.path());
        plan.command.program = "definitely-not-a-real-binary-xyz123".into();
        match execute_command(&plan) {
            Err(PackageError::ExecSpawn { program, .. }) => {
                assert_eq!(program, "definitely-not-a-real-binary-xyz123");
            }
            other => panic!("expected ExecSpawn, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn execute_command_failed_on_nonzero_exit() {
        // `false` is a Unix builtin / coreutil that always exits 1.
        // Windows lacks it, hence the cfg-gate.
        let cwd = tempfile::tempdir().unwrap();
        let mut plan = echo_plan(cwd.path());
        plan.command.program = "false".into();
        plan.command.args = vec![];
        match execute_command(&plan) {
            Err(PackageError::ExecFailed { program, status }) => {
                assert_eq!(program, "false");
                assert!(!status.success());
            }
            other => panic!("expected ExecFailed, got {other:?}"),
        }
    }

    #[test]
    fn format_dry_run_emits_stable_layout() {
        let m = manifest_with(good_artifact());
        let plan = PackagePlan::for_host(
            &m,
            Path::new("/in"),
            Path::new("/out"),
            HostOs::Linux,
            WindowConfig::default(),
        )
        .unwrap();
        let rendered = format_dry_run(&plan);
        // Header / footer braces.
        assert!(rendered.starts_with("PackagePlan {\n"), "got:\n{rendered}");
        assert!(rendered.ends_with("}\n"), "got:\n{rendered}");
        // Field labels appear in stable order.
        let pos = |needle: &str| {
            rendered
                .find(needle)
                .unwrap_or_else(|| panic!("missing {needle}"))
        };
        assert!(pos("source_artifact_dir:") < pos("tauri_src_dist:"));
        assert!(pos("tauri_src_dist:") < pos("output_dir:"));
        assert!(pos("output_dir:") < pos("window:"));
        assert!(pos("window:") < pos("copies (4):"));
        assert!(pos("copies (4):") < pos("command:"));
    }

    #[test]
    fn format_dry_run_lists_copies_in_plan_order() {
        let m = manifest_with(good_artifact());
        let plan = PackagePlan::for_host(
            &m,
            Path::new("/in"),
            Path::new("/out"),
            HostOs::Linux,
            WindowConfig::default(),
        )
        .unwrap();
        let rendered = format_dry_run(&plan);
        // for_host orders: HTML, WASM, boot, host adapter.
        let html_pos = rendered.find("index.html").expect("html");
        let wasm_pos = rendered.find("app_bg.wasm").expect("wasm");
        let boot_pos = rendered.find("boot.js").expect("boot");
        let host_pos = rendered.find("jet-host.js").expect("host");
        assert!(html_pos < wasm_pos);
        assert!(wasm_pos < boot_pos);
        assert!(boot_pos < host_pos);
        // Indices are 1-based + line-prefixed.
        assert!(rendered.contains("    1. /in/index.html"));
        assert!(rendered.contains("    2. /in/app_bg.wasm"));
        assert!(rendered.contains("    3. /in/boot.js"));
        assert!(rendered.contains("    4. /in/jet-host.js"));
    }

    #[test]
    fn format_dry_run_includes_command_and_window() {
        let m = manifest_with(good_artifact());
        let plan = PackagePlan::for_host(
            &m,
            Path::new("/in"),
            Path::new("/out"),
            HostOs::Linux,
            WindowConfig::default().with_title("Cue"),
        )
        .unwrap();
        let rendered = format_dry_run(&plan);
        assert!(
            rendered.contains("command: tauri build (cwd: /out/tauri-src)"),
            "got:\n{rendered}"
        );
        assert!(
            rendered.contains("window: Cue @ 1280x800 (resizable: true)"),
            "got:\n{rendered}"
        );
    }

    #[test]
    fn format_dry_run_handles_empty_args_without_trailing_space() {
        // The args-empty branch of format_dry_run drops the leading
        // space; a `Vec<String>` from `args.join(" ")` would otherwise
        // emit `command: tauri  (cwd: ...)` with a double space.
        let mut plan = echo_plan(Path::new("/cwd"));
        plan.command.program = "true".into();
        plan.command.args = vec![];
        let rendered = format_dry_run(&plan);
        assert!(
            rendered.contains("command: true (cwd: /cwd)"),
            "got:\n{rendered}"
        );
        assert!(!rendered.contains("true  (cwd"), "double-space leak");
    }

    #[test]
    fn command_report_default_is_empty() {
        let r = CommandReport::default();
        assert!(r.program.is_empty());
        assert!(r.args.is_empty());
        assert_eq!(r.working_dir, PathBuf::new());
    }
}
// CODEGEN-END
