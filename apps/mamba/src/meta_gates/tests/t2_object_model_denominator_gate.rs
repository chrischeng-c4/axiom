#![cfg(test)]

// Locks the shape of the T2 Object Model Denominator Gate fixture
// pinned by tests/governance/gates/t2_object_model_denominator/manifest.toml.
// Closes #2700.

use std::fs;
use std::path::PathBuf;

use sha2::{Digest, Sha256};
use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/t2_object_model_denominator/manifest.toml")
}

fn denominator_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/t2_object_model_denominator/denominator.txt")
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(m["version"].as_integer(), Some(1));
    assert_eq!(m["fixture"].as_str(), Some("t2_object_model_denominator"));
    assert_eq!(m["issue"].as_integer(), Some(2644));
    assert_eq!(m["capability"].as_str(), Some("mamba-language-core"));
    assert_eq!(m["work_root_wi"].as_integer(), Some(2030));
    assert_eq!(m["profile"].as_str(), Some("regression-prevention"));
    assert_eq!(m["family"].as_str(), Some("t2_object_model_denominator"));
}

#[test]
fn row_count_matches_denominator_file_lines() {
    let m = manifest();
    let raw = fs::read_to_string(denominator_path()).expect("read denominator");
    let lines: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
    let expected_count = m["row_count"].as_integer().expect("row_count") as usize;
    assert_eq!(
        lines.len(),
        expected_count,
        "row_count in manifest must match line count of denominator.txt"
    );
}

#[test]
fn denominator_sha256_matches_file() {
    let m = manifest();
    let bytes = fs::read(denominator_path()).expect("read denominator bytes");
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hex_digest = format!("{:x}", hasher.finalize());
    let expected_sha256 = m["denominator_sha256"]
        .as_str()
        .expect("denominator_sha256");
    assert_eq!(
        hex_digest, expected_sha256,
        "denominator_sha256 in manifest must match sha256 of denominator.txt"
    );
}
