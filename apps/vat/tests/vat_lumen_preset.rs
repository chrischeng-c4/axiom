use std::fs;

use tempfile::tempdir;
use vat::config::{ServicePreset, load_file};
use vat::lumen_release::normalize_selector;

#[test]
fn lumen_selector_is_exact_release_tag_or_latest() {
    assert_eq!(normalize_selector(None).unwrap(), None);
    assert_eq!(normalize_selector(Some("lumen@0.4.21")).unwrap(), Some("lumen@0.4.21".into()));
    assert!(normalize_selector(Some("0.4.21")).is_err());
}

#[test]
fn config_accepts_native_lumen_and_rejects_container_runtime() {
    let dir = tempdir().unwrap();
    let good = dir.path().join("vat.toml");
    fs::write(&good, "version = 1\n[[services]]\nid = 'search'\npreset = 'lumen'\nversion = 'lumen@0.4.21'\nport = 'auto'\n[[runners]]\nid = 'smoke'\nrequires = ['search']\ncmd = ['true']\n").unwrap();
    let cfg = load_file(&good).unwrap();
    assert_eq!(cfg.service("search").unwrap().preset, Some(ServicePreset::Lumen));
    let bad = dir.path().join("bad.toml");
    fs::write(&bad, "version = 1\n[[services]]\nid = 'search'\npreset = 'lumen'\nruntime = 'docker'\n[[runners]]\nid = 'smoke'\nrequires = ['search']\ncmd = ['true']\n").unwrap();
    assert!(load_file(&bad).unwrap_err().to_string().contains("native-only"));
}
