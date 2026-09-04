//! Shared fixture for `pkgmgr` CLI integration tests.
//!
//! `fixture_pkg` builds a real wheel through the product's `WheelBuilder`
//! (the same path `apps/mamba/tests/pkgmgr/pip.rs`'s `build_wheel` uses)
//! and stages it at `<index>/<pep503-name>/<version>/<file>.whl` beside a
//! `metadata.toml` carrying the `requires` key `load_metadata`
//! (`apps/mamba/src/pkgmanage/lock.rs:398`) reads today. Every case that
//! previously resolved against a metadata-only fixture resolves
//! identically against this one, because resolution only ever reads
//! `metadata.toml`; the wheel is what later lets a synced environment
//! actually import the package.

use std::path::{Path, PathBuf};
use std::process::Command;

use mamba::pkgmanage::pkgmgr::wheel_build::{
    compose_filename, CoreMetadata, WheelBuilder, WheelMetadata,
};
use sha2::{Digest, Sha256};

/// PEP 503 normalize: lowercase, runs of `-`, `_`, `.` collapse to one `-`.
pub fn normalize_pep503(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_sep = false;
    for c in name.chars() {
        let is_sep = c == '-' || c == '_' || c == '.';
        if is_sep {
            if !prev_sep && !out.is_empty() {
                out.push('-');
            }
            prev_sep = true;
        } else {
            out.push(c.to_ascii_lowercase());
            prev_sep = false;
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    out
}

/// The importable module name derived from a package name — matches the
/// convention `apps/mamba/tests/pkgmgr/pip.rs`'s `build_wheel` uses.
pub fn module_name(name: &str) -> String {
    name.replace(['-', '.'], "_").to_ascii_lowercase()
}

/// Build a real, importable wheel for `name`/`version` and stage it in a
/// frozen index directory at `<index>/<pep503-name>/<version>/`, beside a
/// `metadata.toml` naming `requires`. Returns the built wheel's path.
pub fn fixture_pkg(index: &Path, name: &str, version: &str, requires: &[&str]) -> PathBuf {
    let ver_dir = index.join(normalize_pep503(name)).join(version);
    std::fs::create_dir_all(&ver_dir).unwrap();

    let filename = compose_filename(name, version, "py3", "none", "any");
    let mut wheel_meta = WheelMetadata::new("mamba-pkgmgr-test");
    wheel_meta.tags.push("py3-none-any".into());
    let mut core_meta = CoreMetadata::new(name, version);
    core_meta.requires_dist = requires.iter().map(|r| r.to_string()).collect();
    let mut builder = WheelBuilder::new(filename, wheel_meta, core_meta);
    let module = module_name(name);
    builder.add_file(
        format!("{module}/__init__.py"),
        format!("__mamba_fixture__ = {name:?}\n__version__ = {version:?}\n"),
    );
    let wheel_path = builder.build_to_dir(&ver_dir).unwrap();

    let meta = if requires.is_empty() {
        "requires = []\n".to_string()
    } else {
        let arr = requires
            .iter()
            .map(|r| format!("\"{r}\""))
            .collect::<Vec<_>>()
            .join(", ");
        format!("requires = [{arr}]\n")
    };
    std::fs::write(ver_dir.join("metadata.toml"), meta).unwrap();

    wheel_path
}

/// Build a real, importable wheel for `name`/`version` directly in `dir`,
/// with no `metadata.toml` and no `<pep503-name>/<version>` nesting — a
/// bare wheel file the way a direct file/URL install or a `sync` download
/// serves one. The module's `__init__.py` carries `__mamba_fixture__` and
/// `__version__`, the same recipe `fixture_pkg` uses.
pub fn build_wheel_file(dir: &Path, name: &str, version: &str) -> PathBuf {
    std::fs::create_dir_all(dir).unwrap();
    let filename = compose_filename(name, version, "py3", "none", "any");
    let mut wheel_meta = WheelMetadata::new("mamba-pkgmgr-test");
    wheel_meta.tags.push("py3-none-any".into());
    let core_meta = CoreMetadata::new(name, version);
    let mut builder = WheelBuilder::new(filename, wheel_meta, core_meta);
    let module = module_name(name);
    builder.add_file(
        format!("{module}/__init__.py"),
        format!("__mamba_fixture__ = {name:?}\n__version__ = {version:?}\n"),
    );
    builder.build_to_dir(dir).unwrap()
}

/// Read `wheel`'s bytes and mount them at `route` (GET) on `server`.
/// Returns the lowercase hex sha256 of the served bytes — the digest a
/// caller stakes in a lockfile alongside the mounted URL.
pub async fn mount_wheel(server: &wiremock::MockServer, route: &str, wheel: &Path) -> String {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};

    let bytes = std::fs::read(wheel).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let digest = format!("{:x}", hasher.finalize());

    Mock::given(method("GET"))
        .and(path(route))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(bytes))
        .mount(server)
        .await;

    digest
}

/// Like `mount_wheel`, but also requires an exact `authorization` header
/// value — used by the stored-credentials sync case, which must observe
/// that `mamba sync` actually sent the header rather than merely
/// tolerating a response.
pub async fn mount_authed_wheel(
    server: &wiremock::MockServer,
    route: &str,
    wheel: &Path,
    expected_authorization: &str,
) -> String {
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, ResponseTemplate};

    let bytes = std::fs::read(wheel).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let digest = format!("{:x}", hasher.finalize());

    Mock::given(method("GET"))
        .and(path(route))
        .and(header("authorization", expected_authorization))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(bytes))
        .mount(server)
        .await;

    digest
}

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

/// The one new case this issue adds: a fixture wheel installed through
/// the built `mamba` binary's `pip install`, then imported by the
/// `python3` found on `PATH` — proof that a `WheelBuilder`-built wheel is
/// a real, installable, importable artifact, not just bytes that satisfy
/// a sha check.
#[test]
fn fixture_wheel_installs_through_pip_install() {
    let wheel_dir = tempfile::tempdir().unwrap();
    let wheel = build_wheel_file(wheel_dir.path(), "fixture_pip_pkg", "1.0.0");

    let site = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();

    let install = Command::new(mamba_bin())
        .args([
            "pip",
            "install",
            wheel.to_str().unwrap(),
            "--site-packages",
            site.path().to_str().unwrap(),
        ])
        .current_dir(cwd.path())
        .output()
        .expect("spawn mamba pip install");
    assert!(
        install.status.success(),
        "pip install of a fixture wheel must succeed; stdout: {} stderr: {}",
        String::from_utf8_lossy(&install.stdout),
        String::from_utf8_lossy(&install.stderr)
    );

    let module = module_name("fixture_pip_pkg");
    let probe = Command::new("python3")
        .arg("-c")
        .arg(format!(
            "import {module}; print({module}.__mamba_fixture__)"
        ))
        .env("PYTHONPATH", site.path())
        .current_dir(cwd.path())
        .output();
    let probe = match probe {
        Ok(out) => out,
        Err(e) => panic!("python3 not found on PATH, needed for the import probe: {e}"),
    };
    assert!(
        probe.status.success(),
        "import of the installed fixture wheel must succeed; stdout: {} stderr: {}",
        String::from_utf8_lossy(&probe.stdout),
        String::from_utf8_lossy(&probe.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&probe.stdout).trim(),
        "fixture_pip_pkg",
        "installed module must carry the fixture's own name"
    );
}
