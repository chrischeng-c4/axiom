// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-corpus-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests T1..T8 for the jet parity fixture corpus.
//!
//! @spec .aw/tech-design/projects/jet/specs/jet-parity-fixture-corpus.md#TestPlan

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Parser;
use jet_parity_corpus::{
    hash::{hash_bytes, hash_jsx_file},
    manifest::{parse_manifest, CorpusError, ObservationChannel},
    verify::verify,
    FixturesCli,
};

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = projects/jet/parity-corpus → parent thrice = workspace root.
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    here.parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn corpus_root() -> PathBuf {
    workspace_root().join("projects/jet/parity/data/fixtures/mui")
}

fn manifest_path() -> PathBuf {
    corpus_root().join("fixtures.toml")
}

/// Copy the checked-in corpus into a scratch dir so tests can mutate freely.
fn copy_corpus_into(dst: &Path) -> PathBuf {
    let src = corpus_root();
    fs::create_dir_all(dst).expect("create dst");
    copy_dir_recursive(&src, dst);
    dst.join("fixtures.toml")
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    for entry in fs::read_dir(src).expect("read src") {
        let entry = entry.expect("entry");
        let ft = entry.file_type().expect("ft");
        let to = dst.join(entry.file_name());
        if ft.is_dir() {
            fs::create_dir_all(&to).expect("mkdir");
            copy_dir_recursive(&entry.path(), &to);
        } else if ft.is_file() {
            fs::copy(entry.path(), &to).expect("copy");
        }
    }
}

// T1: manifest parses cleanly with three entries.
#[test]
fn t1_manifest_parse_minimal_corpus() {
    let manifest = parse_manifest(&manifest_path()).expect("parse ok");
    assert_eq!(manifest.fixtures.len(), 3, "expected three fixtures");
    let ids: Vec<&str> = manifest.fixtures.iter().map(|f| f.id.as_str()).collect();
    assert!(ids.contains(&"mui-button-primary-v1"));
    assert!(ids.contains(&"mui-textfield-outlined-v1"));
    assert!(ids.contains(&"mui-checkbox-basic-v1"));
    // Every fixture has at least one channel and a non-empty source url.
    for f in &manifest.fixtures {
        assert!(!f.observation_channels.is_empty(), "channels for {}", f.id);
        assert!(!f.mui_demo_source_url.is_empty(), "url for {}", f.id);
        assert_eq!(f.content_hash.len(), 64);
    }
}

// T2: unknown observation_channels values are rejected.
#[test]
fn t2_manifest_rejects_unknown_channel() {
    let tmp = tempfile::tempdir().expect("tmp");
    let path = tmp.path().join("fixtures.toml");
    fs::create_dir_all(tmp.path().join("mui-button-primary-v1")).unwrap();
    fs::write(
        tmp.path().join("mui-button-primary-v1/index.tsx"),
        b"export default function X(){return null;}",
    )
    .unwrap();
    fs::write(
        &path,
        r#"
[[fixtures]]
id = "mui-button-primary-v1"
component = "Button"
jsx_path = "mui-button-primary-v1/index.tsx"
observation_channels = ["pixel", "color-grade"]
mui_demo_source_url = "https://mui.com/material-ui/react-button/"
content_hash = "0000000000000000000000000000000000000000000000000000000000000000"
"#,
    )
    .unwrap();
    let err = parse_manifest(&path).expect_err("should reject unknown channel");
    assert!(
        matches!(err, CorpusError::TomlParse(_)),
        "expected TomlParse, got {err:?}"
    );
}

// T3: malformed ids are rejected.
#[test]
fn t3_manifest_rejects_malformed_id() {
    let tmp = tempfile::tempdir().expect("tmp");
    let path = tmp.path().join("fixtures.toml");
    fs::create_dir_all(tmp.path().join("badid")).unwrap();
    fs::write(tmp.path().join("badid/index.tsx"), b"x").unwrap();
    // Missing -v<n> suffix.
    fs::write(
        &path,
        r#"
[[fixtures]]
id = "mui-button-primary"
component = "Button"
jsx_path = "badid/index.tsx"
observation_channels = ["pixel"]
mui_demo_source_url = "https://example.com"
content_hash = "0000000000000000000000000000000000000000000000000000000000000000"
"#,
    )
    .unwrap();
    let err = parse_manifest(&path).expect_err("malformed");
    assert!(matches!(err, CorpusError::MalformedId(_)), "got {err:?}");

    // Uppercase rejected too.
    fs::write(
        &path,
        r#"
[[fixtures]]
id = "MUI-button-primary-v1"
component = "Button"
jsx_path = "badid/index.tsx"
observation_channels = ["pixel"]
mui_demo_source_url = "https://example.com"
content_hash = "0000000000000000000000000000000000000000000000000000000000000000"
"#,
    )
    .unwrap();
    let err = parse_manifest(&path).expect_err("uppercase");
    assert!(matches!(err, CorpusError::MalformedId(_)), "got {err:?}");
}

// T4: hashing the same file twice yields the same digest.
#[test]
fn t4_hash_jsx_is_deterministic() {
    let jsx = corpus_root().join("mui-button-primary-v1/index.tsx");
    let a = hash_jsx_file(&jsx).expect("hash a");
    let b = hash_jsx_file(&jsx).expect("hash b");
    assert_eq!(a, b);
    assert_eq!(a.len(), 64);
}

// T5: known-vector hash test.
#[test]
fn t5_hash_jsx_matches_known_vector() {
    // sha256("hello") == 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
    let got = hash_bytes(b"hello");
    assert_eq!(
        got,
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}

// T6: verify on the checked-in corpus returns clean.
#[test]
fn t6_verify_clean_corpus() {
    let report = verify(&manifest_path()).expect("verify");
    assert!(report.clean, "drift: {:?}", report.drifted);
    assert_eq!(report.total, 3);
    assert!(report.drifted.is_empty());
}

// T7: mutating one byte of a fixture in a tempdir makes verify exit 1.
#[test]
fn t7_verify_detects_one_byte_jsx_edit() {
    let tmp = tempfile::tempdir().expect("tmp");
    let manifest = copy_corpus_into(tmp.path());
    let target = tmp.path().join("mui-button-primary-v1/index.tsx");
    let mut bytes = fs::read(&target).unwrap();
    // Flip the very last byte (which is a newline → space).
    let last = bytes.len() - 1;
    bytes[last] = bytes[last].wrapping_add(1);
    fs::write(&target, &bytes).unwrap();
    let report = verify(&manifest).expect("verify");
    assert!(!report.clean);
    assert_eq!(report.drifted.len(), 1);
    assert_eq!(report.drifted[0].id, "mui-button-primary-v1");
}

// T8: CLI smoke — list, show, verify.
#[test]
fn t8_cli_list_show_verify_smoke() {
    let manifest = manifest_path();

    // list
    let cli = FixturesCli::try_parse_from([
        "jet-parity-corpus",
        "--manifest",
        manifest.to_str().unwrap(),
        "list",
    ])
    .expect("parse list");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = jet_parity_corpus::cli::dispatch(&cli, &mut out, &mut err).expect("list");
    assert_eq!(code, 0);
    let s = String::from_utf8(out).unwrap();
    assert_eq!(s.lines().count(), 3, "got:\n{s}");
    assert!(s.contains("mui-button-primary-v1"));

    // show
    let cli = FixturesCli::try_parse_from([
        "jet-parity-corpus",
        "--manifest",
        manifest.to_str().unwrap(),
        "show",
        "mui-button-primary-v1",
    ])
    .expect("parse show");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = jet_parity_corpus::cli::dispatch(&cli, &mut out, &mut err).expect("show");
    assert_eq!(code, 0);
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("mui-button-primary-v1"));
    assert!(s.contains("Button"));

    // verify (clean)
    let cli = FixturesCli::try_parse_from([
        "jet-parity-corpus",
        "--manifest",
        manifest.to_str().unwrap(),
        "verify",
    ])
    .expect("parse verify");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = jet_parity_corpus::cli::dispatch(&cli, &mut out, &mut err).expect("verify");
    assert_eq!(code, 0);
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("ok"));

    // verify (drift) via binary invocation against a mutated tempdir copy.
    let tmp = tempfile::tempdir().expect("tmp");
    let mutated = copy_corpus_into(tmp.path());
    let target = tmp.path().join("mui-checkbox-basic-v1/index.tsx");
    let mut bytes = fs::read(&target).unwrap();
    let last = bytes.len() - 1;
    bytes[last] = bytes[last].wrapping_add(1);
    fs::write(&target, bytes).unwrap();

    let exe = env!("CARGO_BIN_EXE_jet-parity-corpus");
    let output = Command::new(exe)
        .args(["--manifest", mutated.to_str().unwrap(), "verify"])
        .output()
        .expect("spawn cli");
    assert_eq!(
        output.status.code(),
        Some(1),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("mui-checkbox-basic-v1"), "stderr: {stderr}");
}

// Sanity: ObservationChannel kebab roundtrip.
#[test]
fn observation_channel_kebab_roundtrip() {
    assert_eq!(ObservationChannel::Pixel.as_kebab(), "pixel");
    assert_eq!(ObservationChannel::AxTree.as_kebab(), "ax-tree");
    assert_eq!(ObservationChannel::FocusOrder.as_kebab(), "focus-order");
    assert_eq!(
        ObservationChannel::PointerHitMap.as_kebab(),
        "pointer-hit-map"
    );
    assert_eq!(ObservationChannel::ImeTrace.as_kebab(), "ime-trace");
}
// CODEGEN-END
