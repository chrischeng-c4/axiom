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

use mamba::pkgmanage::pkgmgr::wheel_build::{
    compose_filename, CoreMetadata, WheelBuilder, WheelMetadata,
};

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
