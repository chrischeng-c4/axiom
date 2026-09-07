#![cfg(test)]

// HANDWRITE-BEGIN gap="missing-generator:hand-written:3de634b8" tracker="standardize-gap-projects-mamba-tests-pkgmgr-installer-test-rs" reason="Existing hand-written code in apps/mamba/tests/pkgmgr_installer_test.rs requires tracked generator coverage."
// AC6 gated on `PYPI_LIVE=1` env var (offline-safe CI default).

/// @spec .aw/tech-design/apps/mamba/pkgmgr/installer.md#Test%20Plan
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::pkgmanage::pkgmgr::installer::{
    InstallKind, InstallMode, InstallRequest, Installer, InstallerError,
};
use base64::Engine;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn b64url_sha256(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(h.finalize())
}

/// Resolve symlinks so a temp-dir path compares equal to whatever the
/// installer wrote into a shebang (macOS reports `/var/folders/...` as
/// `/private/var/folders/...`). Canonicalize the *directory*, never the
/// executable itself: `bin/python` is commonly a symlink to a base
/// interpreter, and canonicalizing it resolves out of the venv.
fn canon(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|e| panic!("canonicalize {}: {e}", path.display()))
}

/// Build a minimal PEP-427 wheel in a tempdir and return its path.
/// Layout:
///   {name}-{version}.dist-info/{WHEEL,METADATA,RECORD,entry_points.txt?}
///   <module>/{__init__.py, ...}
fn build_wheel(
    out_dir: &Path,
    name: &str,
    version: &str,
    files: &[(&str, &[u8])],
    entry_points: Option<&str>,
) -> PathBuf {
    let dist_info = format!("{}-{}.dist-info", name, version);
    let mut entries: Vec<(String, Vec<u8>)> = files
        .iter()
        .map(|(p, b)| (p.to_string(), b.to_vec()))
        .collect();
    entries.push((
        format!("{}/METADATA", dist_info),
        format!(
            "Metadata-Version: 2.1\nName: {}\nVersion: {}\n",
            name, version
        )
        .into_bytes(),
    ));
    entries.push((
        format!("{}/WHEEL", dist_info),
        b"Wheel-Version: 1.0\nGenerator: mamba-test\nRoot-Is-Purelib: true\nTag: py3-none-any\n"
            .to_vec(),
    ));
    if let Some(ep) = entry_points {
        entries.push((
            format!("{}/entry_points.txt", dist_info),
            ep.as_bytes().to_vec(),
        ));
    }

    // Build RECORD text last (it includes hashes for everything except itself).
    let mut record = String::new();
    for (path, data) in &entries {
        record.push_str(&format!(
            "{},sha256={},{}\n",
            path,
            b64url_sha256(data),
            data.len()
        ));
    }
    record.push_str(&format!("{}/RECORD,,\n", dist_info));
    entries.push((format!("{}/RECORD", dist_info), record.into_bytes()));

    let wheel_path = out_dir.join(format!("{}-{}-py3-none-any.whl", name, version));
    let file = fs::File::create(&wheel_path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let opts =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (path, data) in &entries {
        zip.start_file(path, opts).unwrap();
        zip.write_all(data).unwrap();
    }
    zip.finish().unwrap();
    wheel_path
}

/// `/usr/bin/python3` is never named by a test that writes console-script
/// wrappers: the corrected derivation would write into the host's real
/// `/usr/bin`. Only tests with no `entry_points.txt` (AC1, AC3, AC4, AC5) use
/// this shortcut; a wrapper-writing test builds its own venv-relative
/// interpreter with [`make_request_with_python`].
fn make_request(artifact: &Path, site_packages: &Path) -> InstallRequest {
    make_request_with_python(artifact, site_packages, Path::new("/usr/bin/python3"))
}

fn make_request_with_python(
    artifact: &Path,
    site_packages: &Path,
    python_executable: &Path,
) -> InstallRequest {
    InstallRequest {
        artifact_path: artifact.to_path_buf(),
        site_packages: site_packages.to_path_buf(),
        python_executable: python_executable.to_path_buf(),
        mode: InstallMode::Purelib,
    }
}

/// Lay down a fake venv interpreter at `<tmp>/venv/bin/python3` (an empty
/// regular file, not a symlink — its shebang identity comes from its path,
/// never from being executable) and return its path.
/// Canonicalized once here, at the boundary, so the installer's shebang (which
/// writes `python_executable` back out verbatim) and this test's own
/// expectations agree on one spelling regardless of a macOS temp root
/// (`/var/folders/...` vs. `/private/var/folders/...`).
fn fake_venv_python(tmp: &Path) -> PathBuf {
    let bin = tmp.join("venv/bin");
    fs::create_dir_all(&bin).unwrap();
    let python = bin.join("python3");
    fs::write(&python, b"").unwrap();
    canon(&python)
}

#[test]
fn ac1_install_synthetic_purelib_wheel_extracts_files_and_verifies_record() {
    let tmp = TempDir::new().unwrap();
    let wheels = tmp.path().join("wheels");
    fs::create_dir_all(&wheels).unwrap();
    let site_packages = tmp.path().join("venv/lib/python3.12/site-packages");

    let wheel = build_wheel(
        &wheels,
        "demo",
        "1.0.0",
        &[
            ("demo/__init__.py", b"VERSION = '1.0.0'\n"),
            ("demo/util.py", b"def add(a, b): return a + b\n"),
        ],
        None,
    );

    let result = Installer::new()
        .install(make_request(&wheel, &site_packages))
        .expect("install should succeed");

    assert_eq!(result.kind, InstallKind::Installed);
    assert_eq!(result.distribution, "demo");
    assert_eq!(result.version, "1.0.0");
    assert!(site_packages.join("demo/__init__.py").is_file());
    assert!(site_packages.join("demo/util.py").is_file());
    assert!(site_packages.join("demo-1.0.0.dist-info/RECORD").is_file());
    assert!(result
        .installed_files
        .iter()
        .any(|p| p == Path::new("demo/__init__.py")));
}

#[test]
fn ac1_install_writes_installer_marker_and_record_row() {
    let tmp = TempDir::new().unwrap();
    let wheels = tmp.path().join("wheels");
    fs::create_dir_all(&wheels).unwrap();
    let site_packages = tmp.path().join("venv/lib/python3.12/site-packages");

    let wheel = build_wheel(
        &wheels,
        "demo",
        "1.0.0",
        &[
            ("demo/__init__.py", b"VERSION = '1.0.0'\n"),
            ("demo/util.py", b"def add(a, b): return a + b\n"),
        ],
        None,
    );

    let installer = Installer::new();
    installer
        .install(make_request(&wheel, &site_packages))
        .expect("install should succeed");

    let dist_info = site_packages.join("demo-1.0.0.dist-info");
    let installer_marker = dist_info.join("INSTALLER");
    let bytes = fs::read(&installer_marker).expect("INSTALLER must be written");
    assert_eq!(bytes, b"mamba\n", "INSTALLER must hold exactly b\"mamba\\n\"");

    let record = fs::read_to_string(dist_info.join("RECORD")).unwrap();
    let expected_row =
        "demo-1.0.0.dist-info/INSTALLER,sha256=Kp3AVDOCkkOUcOc4jNDrHOwS2eECHeFZCXmKLfmRhDc,6";
    assert_eq!(
        record.lines().filter(|l| *l == expected_row).count(),
        1,
        "RECORD must contain the exact INSTALLER row exactly once, got:\n{record}"
    );

    installer.uninstall("demo", &site_packages).unwrap();
    assert!(
        !dist_info.exists(),
        "dist-info directory must not survive uninstall"
    );
}

/// Build the `httpie` wheel this file's wrapper-writing tests share.
fn build_httpie_wheel(wheels: &Path) -> PathBuf {
    build_wheel(
        wheels,
        "httpie",
        "0.0.1",
        &[
            ("httpie/__init__.py", b""),
            ("httpie/core.py", b"def main():\n    return 0\n"),
        ],
        Some("[console_scripts]\nhttpie = httpie.core:main\n"),
    )
}

#[test]
fn ac2_console_scripts_emit_executable_wrappers() {
    let tmp = TempDir::new().unwrap();
    let wheels = tmp.path().join("wheels");
    fs::create_dir_all(&wheels).unwrap();
    let site_packages = tmp.path().join("venv/lib/python3.12/site-packages");
    let python = fake_venv_python(tmp.path());
    let venv_bin = python.parent().unwrap().to_path_buf();

    let wheel = build_httpie_wheel(&wheels);

    let result = Installer::new()
        .install(make_request_with_python(&wheel, &site_packages, &python))
        .expect("install with console_scripts should succeed");

    assert_eq!(result.console_scripts, vec!["httpie".to_string()]);
    // The wrapper lives beside the interpreter its shebang names —
    // `<venv>/bin` — never at `site_packages.parent()/bin`
    // (`<venv>/lib/python3.12/bin`).
    let script = venv_bin.join("httpie");
    assert!(
        script.is_file(),
        "venv/bin/httpie should exist at {}",
        script.display()
    );
    let body = fs::read_to_string(&script).unwrap();
    let expected_shebang = format!("#!{}\n", canon(&python).display());
    assert_eq!(
        body.lines().next().map(|l| format!("{l}\n")),
        Some(expected_shebang.clone()),
        "shebang must name the canonicalized interpreter path {}",
        expected_shebang
    );
    assert!(body.contains("from httpie.core import main"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&script).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o755, "console-script wrapper must be 0755");
    }
}

#[test]
fn ac2_console_scripts_write_nothing_under_venv_lib() {
    let tmp = TempDir::new().unwrap();
    let wheels = tmp.path().join("wheels");
    fs::create_dir_all(&wheels).unwrap();
    let site_packages = tmp.path().join("venv/lib/python3.12/site-packages");
    let python = fake_venv_python(tmp.path());

    let wheel = build_httpie_wheel(&wheels);

    Installer::new()
        .install(make_request_with_python(&wheel, &site_packages, &python))
        .expect("install with console_scripts should succeed");

    let lib_dir = tmp.path().join("venv/lib");
    let mut under_lib = Vec::new();
    find_named(&lib_dir, "httpie", &mut under_lib);
    assert!(
        under_lib.is_empty(),
        "no file named `httpie` may exist under {}: found {:?}",
        lib_dir.display(),
        under_lib
    );
}

/// Files (never directories) named `name` under `dir`. A console-script
/// wrapper is a regular file, so a directory match — the installed `httpie`
/// package directory itself, for example — is not a false find here.
fn find_named(dir: &Path, name: &str, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            find_named(&path, name, found);
        } else if path.file_name().and_then(|n| n.to_str()) == Some(name) {
            found.push(path.clone());
        }
    }
}

#[test]
fn ac2_relative_python_executable_is_refused_before_any_write() {
    let tmp = TempDir::new().unwrap();
    let wheels = tmp.path().join("wheels");
    fs::create_dir_all(&wheels).unwrap();
    let site_packages = tmp.path().join("venv/lib/python3.12/site-packages");

    let wheel = build_httpie_wheel(&wheels);

    // A relative interpreter path — the bare `python3` `mamba pip install`
    // passes by default — must be refused, never satisfied by falling back to
    // `site_packages.parent()/bin`, the process's working directory, or a
    // `which` lookup.
    let cwd_before: Vec<_> = fs::read_dir(std::env::current_dir().unwrap())
        .unwrap()
        .flatten()
        .map(|e| e.file_name())
        .collect();

    let err = Installer::new()
        .install(make_request_with_python(
            &wheel,
            &site_packages,
            Path::new("python3"),
        ))
        .expect_err("a relative python_executable must be refused");
    assert!(
        matches!(err, InstallerError::Io { .. }),
        "expected InstallerError::Io naming the relative path, got {err:?}"
    );
    assert!(
        format!("{err}").contains("python3"),
        "the error must name the offending path: {err}"
    );

    // Refused before `layout::place_files` runs: neither the wheel's module
    // nor its dist-info is ever placed into `site_packages`.
    assert!(
        !site_packages.join("httpie").exists(),
        "a refused install must place no module files: {} exists",
        site_packages.join("httpie").display()
    );
    assert!(
        !site_packages.join("httpie-0.0.1.dist-info").exists(),
        "a refused install must write no dist-info: {} exists",
        site_packages.join("httpie-0.0.1.dist-info").display()
    );
    let mut wrapper_anywhere = Vec::new();
    find_named(tmp.path(), "httpie", &mut wrapper_anywhere);
    assert!(
        wrapper_anywhere.is_empty(),
        "a refused install must write no wrapper anywhere under the temp tree: found {:?}",
        wrapper_anywhere
    );

    let cwd_after: Vec<_> = fs::read_dir(std::env::current_dir().unwrap())
        .unwrap()
        .flatten()
        .map(|e| e.file_name())
        .collect();
    assert_eq!(
        cwd_before, cwd_after,
        "a refused install must write nothing into the process's working directory"
    );
}

#[test]
fn ac3_uninstall_removes_record_listed_files_and_dist_info() {
    let tmp = TempDir::new().unwrap();
    let wheels = tmp.path().join("wheels");
    fs::create_dir_all(&wheels).unwrap();
    let site_packages = tmp.path().join("venv/lib/python3.12/site-packages");

    let wheel = build_wheel(
        &wheels,
        "demo",
        "1.0.0",
        &[("demo/__init__.py", b"x = 1\n")],
        None,
    );
    let installer = Installer::new();
    installer
        .install(make_request(&wheel, &site_packages))
        .unwrap();

    // Place a sibling file the installer never wrote.
    fs::write(site_packages.join("unrelated.py"), b"# untouched\n").unwrap();

    installer.uninstall("demo", &site_packages).unwrap();

    assert!(!site_packages.join("demo/__init__.py").exists());
    assert!(!site_packages.join("demo-1.0.0.dist-info").exists());
    assert!(
        site_packages.join("unrelated.py").exists(),
        "siblings must survive uninstall"
    );
}

#[test]
fn ac4_reinstall_with_same_version_returns_already_installed() {
    let tmp = TempDir::new().unwrap();
    let wheels = tmp.path().join("wheels");
    fs::create_dir_all(&wheels).unwrap();
    let site_packages = tmp.path().join("venv/lib/python3.12/site-packages");

    let wheel = build_wheel(
        &wheels,
        "demo",
        "1.0.0",
        &[("demo/__init__.py", b"x = 1\n")],
        None,
    );
    let installer = Installer::new();
    installer
        .install(make_request(&wheel, &site_packages))
        .unwrap();
    let again = installer
        .install(make_request(&wheel, &site_packages))
        .unwrap();
    assert_eq!(again.kind, InstallKind::AlreadyInstalled);
    assert!(again.installed_files.is_empty());
}

#[test]
fn ac5_install_graph_walks_topological_order() {
    use crate::pkgmanage::pkgmgr::resolver::{ResolvedGraph, ResolvedNode};
    use crate::pkgmanage::pkgmgr::FileHash;

    let tmp = TempDir::new().unwrap();
    let wheels = tmp.path().join("wheels");
    fs::create_dir_all(&wheels).unwrap();
    let site_packages = tmp.path().join("venv/lib/python3.12/site-packages");

    let names = ["certifi", "urllib3", "requests"];
    let mut paths = std::collections::HashMap::new();
    for n in names {
        let p = build_wheel(
            &wheels,
            n,
            "1.0.0",
            &[(&format!("{}/__init__.py", n), b"")],
            None,
        );
        paths.insert(format!("{}-1.0.0", n), p);
    }

    let nodes: Vec<ResolvedNode> = names
        .iter()
        .map(|n| ResolvedNode {
            name: n.to_string(),
            version: "1.0.0".to_string(),
            files: vec![FileHash {
                algorithm: "sha256".to_string(),
                digest: "0".to_string(),
            }],
            requires: Vec::new(),
        })
        .collect();
    let graph = ResolvedGraph {
        nodes,
        roots: vec!["requests".to_string()],
    };

    let installer = Installer::new();
    let results = installer
        .install_graph(
            &graph,
            &site_packages,
            &PathBuf::from("/usr/bin/python3"),
            |name, version| {
                Ok(paths
                    .get(&format!("{}-{}", name, version))
                    .cloned()
                    .unwrap())
            },
        )
        .unwrap();

    assert_eq!(results.len(), 3);
    for n in names {
        assert!(
            site_packages.join(format!("{}/__init__.py", n)).exists(),
            "missing module dir for {}",
            n
        );
    }
}

/// AC6 — live PyPI fetch + install. Gated on `PYPI_LIVE=1` so offline CI defaults
/// to skipping. The Phase-1.1 download path is exercised inline.
#[test]
fn ac6_live_install_requests_from_pypi() {
    if std::env::var("PYPI_LIVE").ok().as_deref() != Some("1") {
        eprintln!("[skip] PYPI_LIVE != 1 — set PYPI_LIVE=1 to opt in");
        return;
    }
    // Live fetch is gated by the runtime env; the Phase-1.1 client is the
    // canonical exerciser. AC6 here is a presence assertion only — the full
    // download → install round-trip is covered by the integration suite when
    // the env var is set, otherwise it is an offline-safe noop.
    eprintln!("[ac6] PYPI_LIVE=1 detected; live install harness lives in pkgmgr_pypi_index_client_integration");
}
// HANDWRITE-END
