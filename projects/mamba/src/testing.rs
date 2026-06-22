//! Shared #[cfg(test)] helpers for inline manifest schema gates migrated
//! from `tests/*_gate_fixture_*.rs` (refactor: project-mamba/tests, phase 1).
//!
//! `CARGO_MANIFEST_DIR` is identical for `src/**` and `tests/**`, so fixture
//! paths under `tests/cpython/` and `tests/governance/gates/` resolve unchanged.

#![cfg(test)]

use std::fs;
use std::path::PathBuf;
use toml::Value;

pub(crate) fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(crate) fn load_manifest(rel: &str) -> Value {
    let p = manifest_dir().join(rel);
    let raw = fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()));
    raw.parse::<Value>()
        .unwrap_or_else(|e| panic!("parse {}: {e}", p.display()))
}

pub(crate) fn get<'a>(v: &'a Value, key: &str) -> &'a Value {
    v.get(key).unwrap_or_else(|| panic!("missing key: {key}"))
}
pub(crate) fn b(v: &Value, key: &str) -> bool {
    get(v, key)
        .as_bool()
        .unwrap_or_else(|| panic!("{key} not bool"))
}
pub(crate) fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    get(v, key)
        .as_str()
        .unwrap_or_else(|| panic!("{key} not str"))
}
pub(crate) fn i(v: &Value, key: &str) -> i64 {
    get(v, key)
        .as_integer()
        .unwrap_or_else(|| panic!("{key} not int"))
}
pub(crate) fn a<'a>(v: &'a Value, key: &str) -> &'a Vec<Value> {
    get(v, key)
        .as_array()
        .unwrap_or_else(|| panic!("{key} not array"))
}
pub(crate) fn strs<'a>(v: &'a Value, key: &str) -> Vec<&'a str> {
    a(v, key)
        .iter()
        .map(|x| x.as_str().unwrap_or_else(|| panic!("{key}[] not str")))
        .collect()
}
