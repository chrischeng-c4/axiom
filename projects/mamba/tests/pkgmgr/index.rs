//! CLI integration tests for `mamba index build` — materializes local wheels
//! into the frozen-index layout consumed by `mamba add --index` and
//! `mamba lock --index`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

fn run_index_build(workdir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(mamba_bin())
        .arg("index")
        .arg("build")
        .args(args)
        .current_dir(workdir)
        .output()
        .expect("spawn mamba index build")
}

fn run_init(workdir: &Path) -> std::process::Output {
    Command::new(mamba_bin())
        .arg("init")
        .current_dir(workdir)
        .output()
        .expect("spawn mamba init")
}

fn run_add(workdir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(mamba_bin())
        .arg("add")
        .args(args)
        .env_remove("MAMBA_FROZEN_INDEX")
        .env_remove("MAMBA_INDEX_URL")
        .current_dir(workdir)
        .output()
        .expect("spawn mamba add")
}

fn run_lock(workdir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(mamba_bin())
        .arg("lock")
        .args(args)
        .env_remove("MAMBA_FROZEN_INDEX")
        .env_remove("MAMBA_INDEX_URL")
        .current_dir(workdir)
        .output()
        .expect("spawn mamba lock")
}

#[test]
fn index_build_materializes_layout_and_feeds_add_lock() {
    let tmp = tempfile::tempdir().unwrap();
    let wheels = tmp.path().join("wheels");
    std::fs::create_dir(&wheels).unwrap();
    let wheel_a = wheels.join("frozen_index_demo-0.1.0-py3-none-any.whl");
    let wheel_b = wheels.join("frozen_index_demo-0.2.0-py3-none-any.whl");
    std::fs::write(&wheel_a, b"wheel-a").unwrap();
    std::fs::write(&wheel_b, b"wheel-b").unwrap();
    let index = tmp.path().join("index");

    let out = run_index_build(
        tmp.path(),
        &["--out", index.to_str().unwrap(), wheels.to_str().unwrap()],
    );
    assert!(
        out.status.success(),
        "index build must succeed; stdout: {} stderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let indexed_a = index
        .join("frozen-index-demo")
        .join("0.1.0")
        .join("frozen_index_demo-0.1.0-py3-none-any.whl");
    let indexed_b = index
        .join("frozen-index-demo")
        .join("0.2.0")
        .join("frozen_index_demo-0.2.0-py3-none-any.whl");
    assert_eq!(std::fs::read(&indexed_a).unwrap(), b"wheel-a");
    assert_eq!(std::fs::read(&indexed_b).unwrap(), b"wheel-b");

    let replay = run_index_build(
        tmp.path(),
        &["--out", index.to_str().unwrap(), wheels.to_str().unwrap()],
    );
    assert!(
        replay.status.success(),
        "index build replay must succeed; stderr: {}",
        String::from_utf8_lossy(&replay.stderr)
    );
    assert_eq!(
        std::fs::read(&indexed_a).unwrap(),
        b"wheel-a",
        "replay must keep indexed wheel bytes stable"
    );

    let project = tmp.path().join("project");
    std::fs::create_dir(&project).unwrap();
    assert!(run_init(&project).status.success());
    let add = run_add(
        &project,
        &["frozen-index-demo", "--index", index.to_str().unwrap()],
    );
    assert!(
        add.status.success(),
        "generated index must feed add; stderr: {}",
        String::from_utf8_lossy(&add.stderr)
    );
    let manifest = std::fs::read_to_string(project.join("mamba.toml")).unwrap();
    assert!(
        manifest.contains("\"frozen-index-demo==0.2.0\""),
        "add must pick latest generated version: {manifest}"
    );

    let lock = run_lock(&project, &["--index", index.to_str().unwrap()]);
    assert!(
        lock.status.success(),
        "generated index must feed lock; stderr: {}",
        String::from_utf8_lossy(&lock.stderr)
    );
    let lockfile = std::fs::read_to_string(project.join("mamba.lock")).unwrap();
    assert!(
        lockfile.contains("name = \"frozen-index-demo\"")
            && lockfile.contains("version = \"0.2.0\""),
        "lockfile must record generated-index dependency: {lockfile}"
    );
}

#[test]
fn index_build_rejects_malformed_wheel_before_output_mutation() {
    let tmp = tempfile::tempdir().unwrap();
    let wheels = tmp.path().join("wheels");
    std::fs::create_dir(&wheels).unwrap();
    std::fs::write(wheels.join("not-a-valid-wheel.whl"), b"bad").unwrap();
    let index = tmp.path().join("index");

    let out = run_index_build(
        tmp.path(),
        &["--out", index.to_str().unwrap(), wheels.to_str().unwrap()],
    );
    assert!(
        !out.status.success(),
        "malformed wheel name must fail; stdout: {} stderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("parse wheel filename"),
        "diagnostic must name wheel parse failure: {stderr:?}"
    );
    assert!(
        !index.exists(),
        "malformed input must fail before creating output index"
    );
}
