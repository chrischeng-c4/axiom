//! CLI integration tests for `mamba hash` — locks the content-
//! addressed digest primitive that the hash-verified install path
//! (#2686) will consume.
//!
//! Pinned acceptance:
//!
//!   1. `mamba hash <file>` prints `sha256:<hex>  <path>` on a single
//!      line with the pinned SHA-256 of `"hello\n"`.
//!   2. `--algorithm sha384|sha512` switches the digest; algorithm
//!      label and hex length match.
//!   3. Multiple paths produce one line per path, in arg order.
//!   4. Missing file => exit 1 with the filename in stderr; no stdout.
//!   5. Unknown algorithm => exit 1 with "unknown hash algorithm".

use std::path::PathBuf;
use std::process::Command;

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(mamba_bin())
        .args(args)
        .output()
        .expect("spawn mamba")
}

fn write_blob(body: &[u8]) -> tempfile::NamedTempFile {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), body).unwrap();
    tmp
}

#[test]
fn hash_default_sha256_matches_pinned() {
    let blob = write_blob(b"hello\n");
    let out = run(&["hash", blob.path().to_str().unwrap()]);
    assert!(
        out.status.success(),
        "hash must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.starts_with(
            "sha256:5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03  "
        ),
        "wrong sha256 line: {stdout}"
    );
    assert!(stdout.ends_with('\n'), "line ends in newline: {stdout:?}");
}

#[test]
fn hash_sha384_label_and_length() {
    let blob = write_blob(b"hello\n");
    let out = run(&[
        "hash",
        "--algorithm",
        "sha384",
        blob.path().to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let line = stdout.trim_end();
    let (algo_hex, _path) = line.split_once("  ").expect("two-space separator");
    let (algo, hex) = algo_hex.split_once(':').unwrap();
    assert_eq!(algo, "sha384");
    assert_eq!(hex.len(), 96, "sha384 hex length 96; got {}", hex.len());
}

#[test]
fn hash_sha512_label_and_length() {
    let blob = write_blob(b"hello\n");
    let out = run(&["hash", "-a", "sha512", blob.path().to_str().unwrap()]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let line = stdout.trim_end();
    let (algo_hex, _) = line.split_once("  ").unwrap();
    let (algo, hex) = algo_hex.split_once(':').unwrap();
    assert_eq!(algo, "sha512");
    assert_eq!(hex.len(), 128);
}

#[test]
fn hash_multiple_paths_one_line_each() {
    let a = write_blob(b"alpha\n");
    let b = write_blob(b"beta\n");
    let out = run(&[
        "hash",
        a.path().to_str().unwrap(),
        b.path().to_str().unwrap(),
    ]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "two lines for two files: {stdout}");
    assert!(lines[0].contains(a.path().to_str().unwrap()));
    assert!(lines[1].contains(b.path().to_str().unwrap()));
}

#[test]
fn hash_missing_file_fails_with_name_in_stderr() {
    let out = run(&["hash", "/nonexistent/path/does/not/exist.bin"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("/nonexistent/path/does/not/exist.bin"),
        "stderr names the missing file: {stderr:?}"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.is_empty(), "no stdout on failure: {stdout:?}");
}

#[test]
fn hash_unknown_algorithm_fails() {
    let blob = write_blob(b"x");
    let out = run(&["hash", "--algorithm", "md5", blob.path().to_str().unwrap()]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unknown hash algorithm"),
        "stderr signals unknown algo: {stderr:?}"
    );
}
