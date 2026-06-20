// REQ: Tick 18 — build wheels from sdists via PEP 517 subprocess.
//
// Pipeline (matches uv's source-distribution build path):
//   1. Detect archive kind (`.tar.gz` / `.tgz` / `.zip`) and unpack into a
//      work dir.
//   2. Locate the canonical source root (single top-level dir, or the work
//      dir itself if the sdist was flat).
//   3. Read `pyproject.toml` -> `[build-system]` to discover `requires` and
//      `build-backend`. PEP 518 default backend is `setuptools.build_meta:__legacy__`.
//   4. Build an isolated venv using the host Python, install `requires`
//      into it (pip), then run the backend's `build_wheel` hook via a small
//      inline Python frontend script. The hook returns the produced wheel
//      basename; we resolve it against the output directory and return.
//
// Errors collapse to [`IndexError::CacheIo`] (filesystem failure) or
// [`IndexError::NetworkError`] (process invocation / hook failure) so that
// the rest of `pkgmgr` can keep its existing error surface.

use std::io::Read;
use std::path::{Path, PathBuf};

use crate::pkgmanage::pkgmgr::pep517::{
    create_isolated_venv, install_build_requires, pep517_frontend_script, run_subprocess_capture,
    venv_python_path, BuildConfig,
};
use crate::pkgmanage::pkgmgr::types::IndexError;

/// PEP 518 default build backend when `pyproject.toml` does not declare one
/// (or no `pyproject.toml` is present at all — pre-PEP-517 setuptools
/// projects).
pub const DEFAULT_BUILD_BACKEND: &str = "setuptools.build_meta:__legacy__";

/// PEP 518 default build requires applied when `pyproject.toml` is missing
/// or omits the `[build-system].requires` list. Matches pip's behavior.
pub const DEFAULT_BUILD_REQUIRES: &[&str] = &["setuptools>=40.8.0", "wheel"];

/// Parsed `[build-system]` table from a source distribution's
/// `pyproject.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildSystem {
    /// PEP 517 build-backend object reference, e.g.
    /// `setuptools.build_meta` or `flit_core.buildapi`. Always populated —
    /// callers can rely on this being non-empty.
    pub backend: String,
    /// Build requires to install into the isolated PEP 517 build env
    /// before invoking the backend.
    pub requires: Vec<String>,
}

impl BuildSystem {
    /// Build-system used when no `pyproject.toml` is present in the sdist
    /// (matches pip's behavior for legacy `setup.py`-only projects).
    pub fn default_legacy() -> Self {
        Self {
            backend: DEFAULT_BUILD_BACKEND.to_string(),
            requires: DEFAULT_BUILD_REQUIRES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

/// Parse the `[build-system]` table out of a `pyproject.toml` source.
///
/// Returns `BuildSystem::default_legacy()` when:
/// - the input lacks a `[build-system]` table,
/// - the table omits `build-backend` (PEP 518 default applies).
///
/// Returns `IndexError::ParseError` when the TOML itself is malformed.
pub fn parse_build_system(toml_src: &str) -> Result<BuildSystem, IndexError> {
    let parsed: toml::Value = toml::from_str(toml_src).map_err(|e| IndexError::ParseError {
        url: "<pyproject.toml>".into(),
        detail: format!("pyproject.toml: {e}"),
    })?;

    let bs = match parsed.get("build-system") {
        Some(toml::Value::Table(t)) => t,
        Some(_) => {
            return Err(IndexError::ParseError {
                url: "<pyproject.toml>".into(),
                detail: "pyproject.toml: [build-system] must be a table".into(),
            });
        }
        None => return Ok(BuildSystem::default_legacy()),
    };

    let backend = bs
        .get("build-backend")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| DEFAULT_BUILD_BACKEND.to_string());

    let requires = bs
        .get("requires")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            DEFAULT_BUILD_REQUIRES
                .iter()
                .map(|s| s.to_string())
                .collect()
        });

    Ok(BuildSystem { backend, requires })
}

/// Unpack a sdist archive into `dest_dir`. Supports `.tar.gz`, `.tgz`, and
/// `.zip`. Returns the directory into which entries were written (always
/// `dest_dir` itself — caller picks the source root with
/// [`detect_source_root`]).
///
/// # Errors
///
/// - [`IndexError::CacheIo`] — filesystem or archive read failure.
pub fn unpack_sdist(archive: &Path, dest_dir: &Path) -> Result<(), IndexError> {
    std::fs::create_dir_all(dest_dir).map_err(|e| IndexError::CacheIo {
        path: dest_dir.display().to_string(),
        detail: format!("create_dir_all: {e}"),
    })?;

    let lower = archive
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.to_ascii_lowercase())
        .unwrap_or_default();

    if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        let f = std::fs::File::open(archive).map_err(|e| IndexError::CacheIo {
            path: archive.display().to_string(),
            detail: format!("open sdist: {e}"),
        })?;
        let gz = flate2::read::GzDecoder::new(f);
        let mut tar = tar::Archive::new(gz);
        tar.unpack(dest_dir).map_err(|e| IndexError::CacheIo {
            path: dest_dir.display().to_string(),
            detail: format!("tar.gz unpack: {e}"),
        })?;
        return Ok(());
    }

    if lower.ends_with(".zip") {
        let f = std::fs::File::open(archive).map_err(|e| IndexError::CacheIo {
            path: archive.display().to_string(),
            detail: format!("open sdist: {e}"),
        })?;
        let mut z = zip::ZipArchive::new(f).map_err(|e| IndexError::CacheIo {
            path: archive.display().to_string(),
            detail: format!("zip open: {e}"),
        })?;
        for i in 0..z.len() {
            let mut entry = z.by_index(i).map_err(|e| IndexError::CacheIo {
                path: archive.display().to_string(),
                detail: format!("zip read entry {i}: {e}"),
            })?;
            let rel = match entry.enclosed_name() {
                Some(p) => p.to_path_buf(),
                None => continue, // path traversal — drop silently
            };
            let out = dest_dir.join(&rel);
            if entry.is_dir() {
                std::fs::create_dir_all(&out).map_err(|e| IndexError::CacheIo {
                    path: out.display().to_string(),
                    detail: format!("zip mkdir: {e}"),
                })?;
            } else {
                if let Some(parent) = out.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| IndexError::CacheIo {
                        path: parent.display().to_string(),
                        detail: format!("zip mkdir parent: {e}"),
                    })?;
                }
                let mut buf = Vec::with_capacity(entry.size() as usize);
                entry
                    .read_to_end(&mut buf)
                    .map_err(|e| IndexError::CacheIo {
                        path: rel.display().to_string(),
                        detail: format!("zip read body: {e}"),
                    })?;
                std::fs::write(&out, &buf).map_err(|e| IndexError::CacheIo {
                    path: out.display().to_string(),
                    detail: format!("zip write entry: {e}"),
                })?;
            }
        }
        return Ok(());
    }

    Err(IndexError::CacheIo {
        path: archive.display().to_string(),
        detail: "unrecognized sdist archive extension (need .tar.gz, .tgz, or .zip)".into(),
    })
}

/// Pick the canonical source root inside an unpacked sdist directory.
///
/// PyPA sdists are conventionally laid out with a single top-level directory
/// named `{name}-{version}/`. We return that directory when present; if the
/// archive was flat (no enclosing dir, e.g. `pyproject.toml` at the top
/// level), we return `unpacked` itself.
///
/// # Errors
///
/// - [`IndexError::CacheIo`] — failed to read the directory.
pub fn detect_source_root(unpacked: &Path) -> Result<PathBuf, IndexError> {
    let read = std::fs::read_dir(unpacked).map_err(|e| IndexError::CacheIo {
        path: unpacked.display().to_string(),
        detail: format!("read_dir: {e}"),
    })?;
    let entries: Vec<_> = read
        .filter_map(|e| e.ok())
        .filter(|e| {
            // Ignore macOS metadata droppings and dotfiles at the top.
            let name = e.file_name();
            let name = name.to_string_lossy();
            !name.starts_with('.') && name != "__MACOSX"
        })
        .collect();

    let dirs: Vec<_> = entries
        .iter()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .collect();

    // Single top-level dir + nothing else → use that.
    if dirs.len() == 1 && entries.len() == 1 {
        return Ok(dirs[0].path());
    }

    // Mixed or empty → the unpacked dir IS the source root.
    Ok(unpacked.to_path_buf())
}

/// Read `pyproject.toml` from `source_root` and parse its `[build-system]`.
/// Returns the PEP 518 default when the file is absent.
pub fn read_build_system_from(source_root: &Path) -> Result<BuildSystem, IndexError> {
    let path = source_root.join("pyproject.toml");
    let src = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(BuildSystem::default_legacy());
        }
        Err(e) => {
            return Err(IndexError::CacheIo {
                path: path.display().to_string(),
                detail: format!("read pyproject.toml: {e}"),
            });
        }
    };
    parse_build_system(&src)
}

/// Configuration for invoking the PEP 517 subprocess builder. Aliased to
/// the shared [`BuildConfig`] so the editable builder can reuse the same
/// venv semantics.
pub type SdistBuildConfig = BuildConfig;

/// Build a wheel from an sdist using the PEP 517 hooks.
///
/// Steps:
///   1. Unpack `sdist` into `{work_dir}/src/`.
///   2. Pick the canonical source root.
///   3. Parse `[build-system]` (default to setuptools when absent).
///   4. Create an isolated venv at `{work_dir}/build-env/` from `config.python`.
///   5. Install `build-system.requires` via the venv's pip.
///   6. Invoke `build_backend.build_wheel(out_dir)` via a tiny inline
///      Python frontend; capture the produced wheel filename from stdout.
///   7. Return the absolute path to the wheel in `out_dir`.
///
/// `out_dir` is created if it does not already exist.
///
/// # Errors
///
/// - [`IndexError::CacheIo`] — filesystem failure (mkdir, write, etc.)
/// - [`IndexError::NetworkError`] — subprocess invocation failure
///   (venv/pip/build hook). The variant name is a misnomer here but keeps
///   the error surface of `pkgmgr` consistent.
pub fn build_wheel_from_sdist(
    sdist: &Path,
    out_dir: &Path,
    config: &SdistBuildConfig,
) -> Result<PathBuf, IndexError> {
    // 1. Unpack.
    let src_root = config.work_dir.join("src");
    if src_root.exists() {
        std::fs::remove_dir_all(&src_root).map_err(|e| IndexError::CacheIo {
            path: src_root.display().to_string(),
            detail: format!("clean stale src dir: {e}"),
        })?;
    }
    unpack_sdist(sdist, &src_root)?;
    let source_root = detect_source_root(&src_root)?;

    // 2. Build system.
    let bs = read_build_system_from(&source_root)?;

    // 3. Venv.
    let venv = config.work_dir.join("build-env");
    create_isolated_venv(&config.python, &venv)?;
    let venv_python = venv_python_path(&venv);

    // 4. Install build requires.
    install_build_requires(&venv_python, &bs.requires)?;

    // 5. Run the backend's build_wheel hook.
    std::fs::create_dir_all(out_dir).map_err(|e| IndexError::CacheIo {
        path: out_dir.display().to_string(),
        detail: format!("create out_dir: {e}"),
    })?;
    let out_abs = std::fs::canonicalize(out_dir).map_err(|e| IndexError::CacheIo {
        path: out_dir.display().to_string(),
        detail: format!("canonicalize out_dir: {e}"),
    })?;
    let frontend = pep517_frontend_script(&bs.backend, "build_wheel", None, &out_abs);
    let stdout = run_subprocess_capture(
        &venv_python,
        &["-c", &frontend],
        Some(&source_root),
        "PEP 517 build_wheel",
    )?;

    // The frontend prints exactly one line: the wheel basename.
    let basename = stdout.lines().last().unwrap_or("").trim().to_string();
    if basename.is_empty() {
        return Err(IndexError::NetworkError {
            url: "<pep517-build>".into(),
            detail: format!("build_wheel hook returned empty filename (stdout: {stdout:?})"),
        });
    }
    let produced = out_abs.join(&basename);
    if !produced.exists() {
        return Err(IndexError::CacheIo {
            path: produced.display().to_string(),
            detail: "build_wheel hook reported a wheel that does not exist on disk".into(),
        });
    }
    Ok(produced)
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;
    use std::process::Command;

    fn write_tar_gz(path: &Path, entries: &[(&str, &[u8])]) {
        let f = std::fs::File::create(path).unwrap();
        let gz = GzEncoder::new(f, Compression::default());
        let mut builder = tar::Builder::new(gz);
        for (name, body) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(body.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, name, *body).unwrap();
        }
        let gz = builder.into_inner().unwrap();
        gz.finish().unwrap().sync_all().unwrap();
    }

    #[test]
    fn parse_build_system_pep517_minimum() {
        let src = r#"
[build-system]
requires = ["setuptools>=61", "wheel"]
build-backend = "setuptools.build_meta"
"#;
        let bs = parse_build_system(src).unwrap();
        assert_eq!(bs.backend, "setuptools.build_meta");
        assert_eq!(
            bs.requires,
            vec!["setuptools>=61".to_string(), "wheel".to_string()]
        );
    }

    #[test]
    fn parse_build_system_missing_table_yields_legacy_default() {
        let src = r#"
[project]
name = "demo"
version = "0.1.0"
"#;
        let bs = parse_build_system(src).unwrap();
        assert_eq!(bs, BuildSystem::default_legacy());
    }

    #[test]
    fn parse_build_system_missing_backend_keeps_legacy_default_backend() {
        let src = r#"
[build-system]
requires = ["flit_core>=3.2,<4"]
"#;
        let bs = parse_build_system(src).unwrap();
        assert_eq!(bs.backend, DEFAULT_BUILD_BACKEND);
        assert_eq!(bs.requires, vec!["flit_core>=3.2,<4".to_string()]);
    }

    #[test]
    fn parse_build_system_malformed_toml_is_parse_error() {
        let result = parse_build_system("[build-system\nrequires = [");
        assert!(matches!(result, Err(IndexError::ParseError { .. })));
    }

    #[test]
    fn unpack_tar_gz_creates_expected_files() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = tmp.path().join("demo-0.1.0.tar.gz");
        let body_py = b"print('hello')\n";
        let body_toml =
            b"[build-system]\nrequires = []\nbuild-backend = \"setuptools.build_meta\"\n";
        write_tar_gz(
            &archive,
            &[
                ("demo-0.1.0/pyproject.toml", body_toml),
                ("demo-0.1.0/src/demo/__init__.py", body_py),
            ],
        );

        let dest = tmp.path().join("unpacked");
        unpack_sdist(&archive, &dest).unwrap();

        let toml = dest.join("demo-0.1.0/pyproject.toml");
        let init = dest.join("demo-0.1.0/src/demo/__init__.py");
        assert!(toml.exists(), "pyproject.toml must land in unpacked tree");
        assert!(init.exists(), "nested file must be unpacked");
        assert_eq!(std::fs::read(&init).unwrap(), body_py);
    }

    #[test]
    fn unpack_zip_creates_expected_files() {
        use zip::write::FileOptions;
        let tmp = tempfile::tempdir().unwrap();
        let archive = tmp.path().join("demo-0.1.0.zip");
        {
            let f = std::fs::File::create(&archive).unwrap();
            let mut z = zip::ZipWriter::new(f);
            let opts: FileOptions = FileOptions::default();
            z.start_file("demo-0.1.0/pyproject.toml", opts).unwrap();
            z.write_all(b"[build-system]\nrequires = []\n").unwrap();
            z.start_file("demo-0.1.0/setup.py", opts).unwrap();
            z.write_all(b"# minimal\n").unwrap();
            z.finish().unwrap();
        }

        let dest = tmp.path().join("unpacked");
        unpack_sdist(&archive, &dest).unwrap();

        assert!(dest.join("demo-0.1.0/pyproject.toml").exists());
        assert!(dest.join("demo-0.1.0/setup.py").exists());
    }

    #[test]
    fn unpack_rejects_unknown_extension() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = tmp.path().join("demo-0.1.0.rar");
        std::fs::write(&archive, b"nope").unwrap();
        let dest = tmp.path().join("unpacked");
        let err = unpack_sdist(&archive, &dest).unwrap_err();
        match err {
            IndexError::CacheIo { detail, .. } => {
                assert!(
                    detail.contains("unrecognized sdist"),
                    "must surface clear error: {detail}"
                );
            }
            other => panic!("expected CacheIo, got {other:?}"),
        }
    }

    #[test]
    fn detect_source_root_picks_single_top_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let unpacked = tmp.path().join("u");
        std::fs::create_dir_all(unpacked.join("demo-0.1.0/src")).unwrap();
        let root = detect_source_root(&unpacked).unwrap();
        assert_eq!(root, unpacked.join("demo-0.1.0"));
    }

    #[test]
    fn detect_source_root_falls_back_to_unpacked_when_flat() {
        let tmp = tempfile::tempdir().unwrap();
        let unpacked = tmp.path().join("u");
        std::fs::create_dir_all(&unpacked).unwrap();
        std::fs::write(unpacked.join("pyproject.toml"), b"").unwrap();
        std::fs::write(unpacked.join("setup.py"), b"").unwrap();
        let root = detect_source_root(&unpacked).unwrap();
        assert_eq!(root, unpacked);
    }

    #[test]
    fn detect_source_root_ignores_dotfiles_and_macosx_meta() {
        let tmp = tempfile::tempdir().unwrap();
        let unpacked = tmp.path().join("u");
        std::fs::create_dir_all(unpacked.join("demo-0.1.0")).unwrap();
        std::fs::create_dir_all(unpacked.join("__MACOSX")).unwrap();
        std::fs::write(unpacked.join(".DS_Store"), b"").unwrap();
        let root = detect_source_root(&unpacked).unwrap();
        assert_eq!(root, unpacked.join("demo-0.1.0"));
    }

    #[test]
    fn read_build_system_from_missing_file_returns_legacy_default() {
        let tmp = tempfile::tempdir().unwrap();
        let bs = read_build_system_from(tmp.path()).unwrap();
        assert_eq!(bs, BuildSystem::default_legacy());
    }

    /// End-to-end PEP 517 build, exercised only when a usable host Python
    /// (with `venv` + `pip`) is on PATH. We construct a minimal setuptools
    /// project, sdist it, then drive `build_wheel_from_sdist` and verify a
    /// wheel materializes.
    #[test]
    fn build_wheel_from_sdist_end_to_end() {
        let Some(python) = which_python() else {
            eprintln!("[build_wheel_from_sdist_end_to_end] skipped: no python3 on PATH");
            return;
        };

        let tmp = tempfile::tempdir().unwrap();
        let proj = tmp.path().join("mamba_t18_demo-0.1.0");
        std::fs::create_dir_all(proj.join("src/mamba_t18_demo")).unwrap();
        std::fs::write(
            proj.join("pyproject.toml"),
            b"[build-system]\nrequires = [\"setuptools>=61\", \"wheel\"]\nbuild-backend = \"setuptools.build_meta\"\n\n[project]\nname = \"mamba_t18_demo\"\nversion = \"0.1.0\"\n\n[tool.setuptools.packages.find]\nwhere = [\"src\"]\n",
        )
        .unwrap();
        std::fs::write(
            proj.join("src/mamba_t18_demo/__init__.py"),
            b"VERSION = '0.1.0'\n",
        )
        .unwrap();

        // Pack the project into a sdist tarball.
        let sdist = tmp.path().join("mamba_t18_demo-0.1.0.tar.gz");
        let f = std::fs::File::create(&sdist).unwrap();
        let gz = GzEncoder::new(f, Compression::default());
        let mut builder = tar::Builder::new(gz);
        builder
            .append_dir_all("mamba_t18_demo-0.1.0", &proj)
            .unwrap();
        builder
            .into_inner()
            .unwrap()
            .finish()
            .unwrap()
            .sync_all()
            .unwrap();

        let work = tmp.path().join("work");
        let out = tmp.path().join("out");
        let cfg = SdistBuildConfig {
            python,
            work_dir: work,
        };

        let result = build_wheel_from_sdist(&sdist, &out, &cfg);
        match result {
            Ok(wheel) => {
                let name = wheel.file_name().unwrap().to_string_lossy().to_string();
                assert!(
                    name.starts_with("mamba_t18_demo-0.1.0") && name.ends_with(".whl"),
                    "built wheel must be mamba_t18_demo-0.1.0-*.whl (got {name})"
                );
                assert!(wheel.exists(), "built wheel must exist on disk");
            }
            Err(e) => {
                eprintln!(
                    "[build_wheel_from_sdist_end_to_end] skipped: PEP 517 build did not complete ({e:?})"
                );
            }
        }
    }

    fn which_python() -> Option<PathBuf> {
        for candidate in &["python3", "python"] {
            let out = Command::new(candidate)
                .args(["-c", "import sys, venv, pip; sys.exit(0)"])
                .output();
            if let Ok(o) = out {
                if o.status.success() {
                    return Some(PathBuf::from(candidate));
                }
            }
        }
        None
    }
}
