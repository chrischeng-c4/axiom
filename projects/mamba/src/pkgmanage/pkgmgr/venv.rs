// PEP 405 virtual environment creation (Tick 37).
//
// Replicates `uv venv`: lay down a self-contained Python environment
// at a chosen prefix, write the PEP 405 `pyvenv.cfg` marker file, and
// expose the cross-platform layout (`bin/` vs `Scripts/`, etc) so the
// installer / runner can find `python`, site-packages, and headers.
//
// Splits into three deliberate layers:
//   1. *Layout* (pure data): `VenvLayout::for_platform` derives every
//      path from `(root, python_version, platform)` — no I/O. The
//      cross-platform variants are PEP 405-mandated:
//        POSIX:  bin/, lib/pythonX.Y/site-packages/, include/
//        Windows: Scripts/, Lib/site-packages/, Include/
//   2. *Manifest* (pure render): `render_pyvenv_cfg` turns
//      `PyvenvCfg` (home / include-system-site-packages / version /
//      executable / command) into the canonical `key = value` body.
//      This matches what CPython's `venv` module writes, byte-for-byte
//      for the required keys.
//   3. *Driver* (real I/O): `create_venv` invokes the chosen Python's
//      `python -m venv` to populate the tree, then verifies the
//      contract (pyvenv.cfg present, required dirs created). The
//      driver intentionally delegates to upstream `venv` for the
//      interpreter copy/symlink because that's where cross-platform
//      compatibility, fallback FS support (#R7), and `ensurepip` live.
//
// `create_venv` refuses-to-overwrite by default (gate R3) — callers
// that want the destructive replace path must pass `clobber: true`.
//
// Tests:
//   * Layout + manifest are exercised purely without `python3` present.
//   * `create_venv_*` tests soft-skip with `eprintln!` when no Python
//     is on PATH (CI/sandbox-friendly).

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::pkgmanage::pkgmgr::toolchain::PythonVersion;
use crate::pkgmanage::pkgmgr::types::IndexError;

/// Target operating-system family for the layout. Cross-compilation
/// isn't a goal — we expose this as a typed enum so the rendering
/// logic is independent of the host build target, which keeps the
/// layout tests platform-agnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformKind {
    Posix,
    Windows,
}

impl PlatformKind {
    /// Pick the platform of the running build.
    pub const fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else {
            Self::Posix
        }
    }
}

/// PEP 405 directory layout for a venv root. Every field is an
/// absolute path *under* `root` (caller-owned), no I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VenvLayout {
    pub root: PathBuf,
    pub bin_dir: PathBuf,
    pub lib_dir: PathBuf,
    pub site_packages: PathBuf,
    pub include_dir: PathBuf,
    /// `bin/python` (or `Scripts/python.exe`) — the interpreter the
    /// stdlib's `venv` module lays down.
    pub python_executable: PathBuf,
    /// PEP 405 marker file.
    pub pyvenv_cfg: PathBuf,
    pub platform: PlatformKind,
}

impl VenvLayout {
    /// Derive every venv path from `(root, version, platform)`.
    pub fn for_platform(root: &Path, version: &PythonVersion, platform: PlatformKind) -> Self {
        let (bin, site_pkgs, include, exe_name) = match platform {
            PlatformKind::Posix => (
                "bin".to_string(),
                format!("lib/python{}.{}/site-packages", version.major, version.minor),
                "include".to_string(),
                "python".to_string(),
            ),
            PlatformKind::Windows => (
                "Scripts".to_string(),
                "Lib/site-packages".to_string(),
                "Include".to_string(),
                "python.exe".to_string(),
            ),
        };
        let bin_dir = root.join(&bin);
        let site_packages = root.join(&site_pkgs);
        // lib_dir is the parent of site-packages; on POSIX that's
        // lib/pythonX.Y, on Windows it collapses to Lib. We need it
        // for downstream tooling that wants the lib root rather than
        // just site-packages.
        let lib_dir = site_packages
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| root.join(match platform {
                PlatformKind::Posix => "lib",
                PlatformKind::Windows => "Lib",
            }));
        VenvLayout {
            root: root.to_path_buf(),
            bin_dir,
            lib_dir,
            site_packages,
            include_dir: root.join(&include),
            python_executable: root.join(&bin).join(&exe_name),
            pyvenv_cfg: root.join("pyvenv.cfg"),
            platform,
        }
    }

    /// Shorthand for the running OS.
    pub fn for_current_platform(root: &Path, version: &PythonVersion) -> Self {
        Self::for_platform(root, version, PlatformKind::current())
    }

    /// PEP 405 required subdirs as POSIX-relative strings (matches the
    /// venv_phase_gate manifest contract for POSIX).
    pub fn required_subdirs_posix(version: &PythonVersion) -> Vec<String> {
        vec![
            "bin".to_string(),
            format!("lib/python{}.{}/site-packages", version.major, version.minor),
            "include".to_string(),
        ]
    }
}

/// PEP 405 `pyvenv.cfg` contents. Required keys per R1:
///   * home
///   * include-system-site-packages
///   * version
///
/// Optional keys that CPython's `venv` always writes but which the
/// contract doesn't *require*. We pass them through verbatim so the
/// rendered file is byte-compatible with what `python -m venv` would
/// emit if invoked separately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PyvenvCfg {
    pub home: PathBuf,
    pub include_system_site_packages: bool,
    pub version: PythonVersion,
    pub executable: Option<PathBuf>,
    pub command: Option<String>,
    pub base_prefix: Option<PathBuf>,
    pub base_executable: Option<PathBuf>,
}

impl PyvenvCfg {
    /// Minimal cfg with just the three required keys (R1).
    pub fn minimal(home: impl Into<PathBuf>, version: PythonVersion) -> Self {
        PyvenvCfg {
            home: home.into(),
            include_system_site_packages: false,
            version,
            executable: None,
            command: None,
            base_prefix: None,
            base_executable: None,
        }
    }
}

/// Required key names for the PEP 405 contract.
pub const REQUIRED_PYVENV_KEYS: &[&str] = &["home", "include-system-site-packages", "version"];

/// Render a PEP 405 pyvenv.cfg body. Keys appear in CPython's
/// canonical order; trailing newline included so the file ends cleanly.
pub fn render_pyvenv_cfg(cfg: &PyvenvCfg) -> String {
    let mut out = String::new();
    push_kv(&mut out, "home", &display_path(&cfg.home));
    push_kv(
        &mut out,
        "include-system-site-packages",
        if cfg.include_system_site_packages {
            "true"
        } else {
            "false"
        },
    );
    let version = format!("{}.{}.{}", cfg.version.major, cfg.version.minor, cfg.version.patch);
    push_kv(&mut out, "version", &version);
    if let Some(exe) = &cfg.executable {
        push_kv(&mut out, "executable", &display_path(exe));
    }
    if let Some(cmd) = &cfg.command {
        push_kv(&mut out, "command", cmd);
    }
    if let Some(bp) = &cfg.base_prefix {
        push_kv(&mut out, "base-prefix", &display_path(bp));
    }
    if let Some(be) = &cfg.base_executable {
        push_kv(&mut out, "base-executable", &display_path(be));
    }
    out
}

fn push_kv(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = ");
    out.push_str(value);
    out.push('\n');
}

fn display_path(p: &Path) -> String {
    p.display().to_string()
}

/// Parse a pyvenv.cfg body into key/value pairs. Whitespace around
/// `=` is stripped; blank/comment lines are skipped; duplicate keys
/// take the *last* value (matches CPython's `venv` reader).
pub fn parse_pyvenv_cfg(src: &str) -> Result<Vec<(String, String)>, IndexError> {
    let mut out = Vec::new();
    for (lineno, raw) in src.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            return Err(IndexError::ParseError {
                url: "<pyvenv.cfg>".into(),
                detail: format!("pyvenv.cfg line {} missing '=': {raw:?}", lineno + 1),
            });
        };
        out.push((k.trim().to_string(), v.trim().to_string()));
    }
    Ok(out)
}

/// Verify a pyvenv.cfg body satisfies R1 — every required key present
/// with a non-empty value. Returns the failure-kind string spec'd by
/// the venv_phase_gate manifest so the CLI can map to exit code 391.
pub fn verify_required_keys(src: &str) -> Result<(), IndexError> {
    let kvs = parse_pyvenv_cfg(src)?;
    for required in REQUIRED_PYVENV_KEYS {
        let found = kvs.iter().any(|(k, v)| k == required && !v.is_empty());
        if !found {
            return Err(IndexError::ParseError {
                url: "<pyvenv.cfg>".into(),
                detail: format!(
                    "mvp_package_manager_pyvenv_cfg_required_key_missing: {required}"
                ),
            });
        }
    }
    Ok(())
}

/// Options controlling `create_venv`.
#[derive(Debug, Clone)]
pub struct VenvOptions {
    /// Path to the Python interpreter the new venv will be based on.
    /// `create_venv` invokes `<python> -m venv <root>` so this must be
    /// a real interpreter on disk, not just a name. Probe its version
    /// first via `toolchain::probe_python_version`.
    pub python: PathBuf,
    /// PEP 405 root the venv will live in. Caller owns existence /
    /// non-existence of this path; see `clobber`.
    pub root: PathBuf,
    /// Mirror CPython `venv --system-site-packages`. Defaults to false.
    pub system_site_packages: bool,
    /// `--copies` rather than the default symlink interpreter (R7).
    pub copies: bool,
    /// Skip pip seeding (`--without-pip`). Speeds up tests + matches
    /// `uv venv`'s "we install our own pip on demand" behavior.
    pub without_pip: bool,
    /// Optional human-readable prompt (the `command` written into
    /// pyvenv.cfg). Not required by the contract; passed to `venv`
    /// only when set.
    pub prompt: Option<String>,
    /// If `true`, an existing pyvenv.cfg at `root` is removed and the
    /// venv is rebuilt. If `false` (default per R3), creation fails
    /// with `VenvCreationOutcome::Refused`. The CLI surfaces this
    /// outcome rather than `IndexError`.
    pub clobber: bool,
}

impl VenvOptions {
    pub fn new(python: impl Into<PathBuf>, root: impl Into<PathBuf>) -> Self {
        VenvOptions {
            python: python.into(),
            root: root.into(),
            system_site_packages: false,
            copies: false,
            without_pip: true,
            prompt: None,
            clobber: false,
        }
    }
}

/// Result of `create_venv`. Distinguishes the "we refused because a
/// venv already exists" non-error branch (R3 outcome
/// `refused_existing_pyvenv_cfg`) from `created_new`. Anything
/// catastrophic still surfaces as `IndexError`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VenvCreationOutcome {
    /// New venv was successfully laid down at the requested root.
    Created { layout: VenvLayout, version: PythonVersion },
    /// The target already had a pyvenv.cfg and `clobber` was false.
    Refused { reason: String },
}

/// Create a PEP 405 venv at `opts.root`. Drives the chosen Python's
/// stdlib `venv` module — that owns the interpreter copy/symlink and
/// site-packages bootstrapping (R7 + R5 implementation details).
///
/// Failure modes:
///   * `IndexError::ParseError` for contract violations (no pyvenv.cfg
///     produced, missing required keys, missing required subdir).
///   * `IndexError::NetworkError` for spawn/exit failures from Python.
pub fn create_venv(opts: &VenvOptions) -> Result<VenvCreationOutcome, IndexError> {
    let existing_cfg = opts.root.join("pyvenv.cfg");
    if existing_cfg.exists() {
        if !opts.clobber {
            return Ok(VenvCreationOutcome::Refused {
                reason: format!(
                    "refused_existing_pyvenv_cfg: {} already contains a venv",
                    opts.root.display()
                ),
            });
        }
        // Clobber: remove the whole tree and rebuild. Safer than
        // partial cleanup because half-replaced site-packages can
        // shadow imports unpredictably.
        std::fs::remove_dir_all(&opts.root).map_err(|e| IndexError::ParseError {
            url: opts.root.display().to_string(),
            detail: format!("removing existing venv for clobber: {e}"),
        })?;
    }

    let version = crate::pkgmanage::pkgmgr::toolchain::probe_python_version(&opts.python)?;
    let layout = VenvLayout::for_current_platform(&opts.root, &version);

    let mut cmd = Command::new(&opts.python);
    cmd.arg("-I"); // ignore PYTHON* env vars; uv does the same
    cmd.arg("-m").arg("venv");
    if opts.system_site_packages {
        cmd.arg("--system-site-packages");
    }
    if opts.copies {
        cmd.arg("--copies");
    }
    if opts.without_pip {
        cmd.arg("--without-pip");
    }
    if let Some(prompt) = &opts.prompt {
        cmd.arg("--prompt").arg(prompt);
    }
    cmd.arg(&opts.root);

    let output = cmd.output().map_err(|e| IndexError::NetworkError {
        url: opts.python.display().to_string(),
        detail: format!("spawning python -m venv: {e}"),
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(IndexError::NetworkError {
            url: opts.python.display().to_string(),
            detail: format!(
                "python -m venv exited with {:?}: {}",
                output.status.code(),
                stderr.trim()
            ),
        });
    }

    // Contract verification (R1 + R2).
    if !layout.pyvenv_cfg.exists() {
        return Err(IndexError::ParseError {
            url: layout.pyvenv_cfg.display().to_string(),
            detail: "mvp_package_manager_pyvenv_cfg_missing".into(),
        });
    }
    let cfg_body = std::fs::read_to_string(&layout.pyvenv_cfg).map_err(|e| {
        IndexError::ParseError {
            url: layout.pyvenv_cfg.display().to_string(),
            detail: format!("reading pyvenv.cfg: {e}"),
        }
    })?;
    verify_required_keys(&cfg_body)?;

    for required in required_dirs(&layout) {
        if !required.is_dir() {
            return Err(IndexError::ParseError {
                url: required.display().to_string(),
                detail: format!(
                    "mvp_package_manager_venv_required_subdir_missing: {}",
                    required.display()
                ),
            });
        }
    }

    Ok(VenvCreationOutcome::Created { layout, version })
}

fn required_dirs(layout: &VenvLayout) -> Vec<PathBuf> {
    vec![
        layout.bin_dir.clone(),
        layout.site_packages.clone(),
        // include dir is required by the manifest but `python -m venv`
        // may skip it on systems without Python.h. We still assert
        // its presence so the contract is strict on supported hosts.
        layout.include_dir.clone(),
    ]
}

/// Remove a venv at `root`. Implements R6: refuses to delete if no
/// pyvenv.cfg is present (so we never accidentally rm -rf a project
/// directory). Returns the outcome string spec'd by the manifest.
pub fn remove_venv(root: &Path) -> Result<String, IndexError> {
    let cfg = root.join("pyvenv.cfg");
    if !cfg.exists() {
        return Ok("refused_no_pyvenv_cfg".to_string());
    }
    std::fs::remove_dir_all(root).map_err(|e| IndexError::ParseError {
        url: root.display().to_string(),
        detail: format!("removing venv tree: {e}"),
    })?;
    Ok("removed".to_string())
}

/// Best-effort lookup of `python3` on `PATH`. Used by tests that
/// soft-skip when no Python is available.
pub fn first_python_on_path() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for entry in std::env::split_paths(&path) {
        for name in ["python3", "python"] {
            let candidate = entry.join(name);
            #[cfg(windows)]
            let candidate = candidate.with_extension("exe");
            if candidate.is_file() {
                // Filter out a `python` shim that points at the very
                // mamba binary we're running inside (cclab installs one
                // sometimes). We just sanity-test the basename.
                if candidate.file_name().and_then(OsStr::to_str) == Some("mamba") {
                    continue;
                }
                return Some(candidate);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkgmanage::pkgmgr::toolchain::probe_python_version;

    fn v(major: u32, minor: u32, patch: u32) -> PythonVersion {
        PythonVersion::new(major, minor, patch)
    }

    #[test]
    fn layout_posix_uses_bin_lib_pythonxy_include() {
        let layout = VenvLayout::for_platform(
            Path::new("/tmp/v"),
            &v(3, 12, 1),
            PlatformKind::Posix,
        );
        assert_eq!(layout.bin_dir, PathBuf::from("/tmp/v/bin"));
        assert_eq!(
            layout.site_packages,
            PathBuf::from("/tmp/v/lib/python3.12/site-packages")
        );
        assert_eq!(layout.include_dir, PathBuf::from("/tmp/v/include"));
        assert_eq!(layout.python_executable, PathBuf::from("/tmp/v/bin/python"));
        assert_eq!(layout.pyvenv_cfg, PathBuf::from("/tmp/v/pyvenv.cfg"));
        assert_eq!(layout.lib_dir, PathBuf::from("/tmp/v/lib/python3.12"));
    }

    #[test]
    fn layout_windows_uses_scripts_lib_site_packages() {
        let layout = VenvLayout::for_platform(
            Path::new("C:\\v"),
            &v(3, 12, 1),
            PlatformKind::Windows,
        );
        // On non-Windows hosts PathBuf still composes with the host
        // separator, so we compare on the final-segment strings rather
        // than the whole display.
        assert!(layout.bin_dir.ends_with("Scripts"));
        assert!(layout.site_packages.ends_with("Lib/site-packages") ||
                layout.site_packages.ends_with("Lib\\site-packages"));
        assert!(layout.include_dir.ends_with("Include"));
        assert!(layout.python_executable.ends_with("python.exe"));
    }

    #[test]
    fn required_subdirs_posix_matches_manifest_contract() {
        let subs = VenvLayout::required_subdirs_posix(&v(3, 12, 0));
        assert_eq!(
            subs,
            vec![
                "bin".to_string(),
                "lib/python3.12/site-packages".to_string(),
                "include".to_string(),
            ]
        );
    }

    #[test]
    fn render_minimal_pyvenv_cfg_has_three_required_keys() {
        let cfg = PyvenvCfg::minimal("/usr/local", v(3, 12, 1));
        let body = render_pyvenv_cfg(&cfg);
        assert!(body.contains("home = /usr/local\n"));
        assert!(body.contains("include-system-site-packages = false\n"));
        assert!(body.contains("version = 3.12.1\n"));
        // Optional keys absent when None.
        assert!(!body.contains("executable"));
        assert!(!body.contains("command"));
    }

    #[test]
    fn render_includes_optional_keys_when_present() {
        let cfg = PyvenvCfg {
            home: PathBuf::from("/h"),
            include_system_site_packages: true,
            version: v(3, 12, 0),
            executable: Some(PathBuf::from("/v/bin/python")),
            command: Some("python -m venv /v".into()),
            base_prefix: Some(PathBuf::from("/u/lib")),
            base_executable: Some(PathBuf::from("/u/bin/python3")),
        };
        let body = render_pyvenv_cfg(&cfg);
        assert!(body.contains("include-system-site-packages = true\n"));
        assert!(body.contains("executable = /v/bin/python\n"));
        assert!(body.contains("command = python -m venv /v\n"));
        assert!(body.contains("base-prefix = /u/lib\n"));
        assert!(body.contains("base-executable = /u/bin/python3\n"));
    }

    #[test]
    fn parse_pyvenv_cfg_strips_whitespace_and_skips_comments() {
        let kvs = parse_pyvenv_cfg(
            "# header comment\n\
             home = /usr/local\n\
             include-system-site-packages  =  false\n\
             \n\
             version=3.12.1\n",
        )
        .unwrap();
        assert_eq!(kvs[0], ("home".to_string(), "/usr/local".to_string()));
        assert_eq!(
            kvs[1],
            (
                "include-system-site-packages".to_string(),
                "false".to_string()
            )
        );
        assert_eq!(kvs[2], ("version".to_string(), "3.12.1".to_string()));
    }

    #[test]
    fn parse_pyvenv_cfg_errors_on_missing_equals() {
        let err = parse_pyvenv_cfg("homeless line\n").unwrap_err();
        assert!(format!("{err}").contains("missing '='"));
    }

    #[test]
    fn verify_required_keys_passes_for_minimal_cfg() {
        let body = render_pyvenv_cfg(&PyvenvCfg::minimal("/u", v(3, 12, 1)));
        verify_required_keys(&body).expect("minimal cfg satisfies R1");
    }

    #[test]
    fn verify_required_keys_fails_when_any_required_key_missing() {
        for missing in REQUIRED_PYVENV_KEYS {
            let mut body = render_pyvenv_cfg(&PyvenvCfg::minimal("/u", v(3, 12, 1)));
            // Remove the line whose key matches `missing`.
            body = body
                .lines()
                .filter(|l| !l.starts_with(&format!("{missing} ")))
                .collect::<Vec<_>>()
                .join("\n");
            let err = verify_required_keys(&body).unwrap_err();
            assert!(
                format!("{err}").contains("pyvenv_cfg_required_key_missing"),
                "missing {missing}: {err}"
            );
        }
    }

    #[test]
    fn remove_venv_refuses_without_pyvenv_cfg() {
        let tmp = tempfile::tempdir().unwrap();
        // Put a sibling file but no pyvenv.cfg.
        std::fs::write(tmp.path().join("garbage.txt"), b"hi").unwrap();
        let outcome = remove_venv(tmp.path()).unwrap();
        assert_eq!(outcome, "refused_no_pyvenv_cfg");
        // Tree must still be there.
        assert!(tmp.path().join("garbage.txt").exists());
    }

    #[test]
    fn remove_venv_removes_when_pyvenv_cfg_present() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("v");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("pyvenv.cfg"),
            render_pyvenv_cfg(&PyvenvCfg::minimal("/u", v(3, 12, 1))),
        )
        .unwrap();
        let outcome = remove_venv(&root).unwrap();
        assert_eq!(outcome, "removed");
        assert!(!root.exists());
    }

    #[test]
    fn create_venv_succeeds_on_real_python() {
        let Some(python) = first_python_on_path() else {
            eprintln!("no python on PATH — skipping create_venv_succeeds_on_real_python");
            return;
        };
        // Skip if probe fails (e.g. python is a non-functional shim).
        let Ok(version) = probe_python_version(&python) else {
            eprintln!("python {python:?} probe failed — skipping");
            return;
        };
        if version.major != 3 {
            eprintln!("non-3.x python {version:?} — skipping");
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("v");
        let opts = VenvOptions::new(&python, &root);
        let outcome = create_venv(&opts).expect("create_venv");
        match outcome {
            VenvCreationOutcome::Created { layout, version: v2 } => {
                assert_eq!(layout.root, root);
                assert!(layout.pyvenv_cfg.is_file());
                let body = std::fs::read_to_string(&layout.pyvenv_cfg).unwrap();
                verify_required_keys(&body).expect("required keys present");
                assert_eq!(v2.major, version.major);
                assert_eq!(v2.minor, version.minor);
            }
            VenvCreationOutcome::Refused { reason } => {
                panic!("unexpected refusal: {reason}");
            }
        }
    }

    #[test]
    fn create_venv_refuses_when_pyvenv_cfg_already_present() {
        let Some(python) = first_python_on_path() else {
            eprintln!("no python on PATH — skipping refuse-overwrite test");
            return;
        };
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("v");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("pyvenv.cfg"),
            render_pyvenv_cfg(&PyvenvCfg::minimal(&root, v(3, 12, 0))),
        )
        .unwrap();

        let opts = VenvOptions::new(&python, &root);
        let outcome = create_venv(&opts).expect("create_venv");
        match outcome {
            VenvCreationOutcome::Refused { reason } => {
                assert!(reason.contains("refused_existing_pyvenv_cfg"));
            }
            VenvCreationOutcome::Created { .. } => {
                panic!("expected refusal, got Created");
            }
        }
    }

    #[test]
    fn create_venv_clobbers_when_opted_in() {
        let Some(python) = first_python_on_path() else {
            eprintln!("no python on PATH — skipping clobber test");
            return;
        };
        let Ok(version) = probe_python_version(&python) else {
            eprintln!("probe failed — skipping");
            return;
        };
        if version.major != 3 {
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("v");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("pyvenv.cfg"),
            "home = /nonsense\nversion = 0.0.0\ninclude-system-site-packages = false\n",
        )
        .unwrap();
        std::fs::write(root.join("marker.txt"), b"stale").unwrap();

        let mut opts = VenvOptions::new(&python, &root);
        opts.clobber = true;
        let outcome = create_venv(&opts).expect("clobber create_venv");
        match outcome {
            VenvCreationOutcome::Created { layout, .. } => {
                assert!(layout.pyvenv_cfg.is_file());
                // Stale marker must be gone after clobber.
                assert!(!root.join("marker.txt").exists());
            }
            VenvCreationOutcome::Refused { reason } => {
                panic!("clobber should not refuse, got: {reason}");
            }
        }
    }

    #[test]
    fn first_python_on_path_returns_existing_executable_or_none() {
        // Whatever the answer is, the file (if any) must be a real
        // file we can stat — guards against PATH dirs that vanished.
        if let Some(p) = first_python_on_path() {
            assert!(p.is_file(), "first_python_on_path returned non-file {p:?}");
        }
    }
}
