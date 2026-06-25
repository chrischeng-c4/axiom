// `mamba pkgmgr-validate` — drive the release-blocking
// package-manager workflow families and emit a JSON summary that
// matches validation/profiles/package_manager.toml [runner_contract]
// + [summary].
//
// Acceptance (validation/profiles/package_manager.toml #2816):
//
//   - Profile fails on any required offline workflow failure.
//   - Live network tests are opt-in (`--include-live-network`),
//     bucketed under `optional`, and never block.
//   - Summary names project_path, lockfile_path, environment_path
//     per family.
//   - JSON output keys per family: passed, failed, missing, fixtures.
//
// The runner spawns the current `mamba` binary against ephemeral
// tempdirs so it exercises the same CLI surface end-users hit.

use anyhow::{Context, Result};
use clap::ArgMatches;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Lightweight self-cleaning tempdir — avoids pulling the `tempfile`
/// dev-dep into the runtime lib.
struct ScratchDir(PathBuf);

impl ScratchDir {
    fn new(prefix: &str) -> Self {
        let pid = std::process::id();
        let n = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("mamba-{prefix}-{pid}-{nanos}-{n}"));
        std::fs::create_dir_all(&path).expect("create scratch dir");
        ScratchDir(path)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const REQUIRED_FAMILIES: &[&str] = &[
    "init",
    "auth",
    "index",
    "add",
    "lock",
    "export",
    "tree",
    "version",
    "pip",
    "venv",
    "python",
    "workspace",
    "shell",
    "sync",
    "run",
    "install",
    "tool",
    "hash",
    "cache",
];

pub fn cmd_validate(sub: &ArgMatches) -> Result<()> {
    let include_live = sub.get_flag("include-live-network");
    let json_out = sub.get_flag("json");
    let bin = std::env::current_exe().context("locate current mamba binary")?;

    let mut results: BTreeMap<String, FamilyResult> = BTreeMap::new();
    for fam in REQUIRED_FAMILIES {
        let result = run_family(fam, &bin);
        results.insert((*fam).to_string(), result);
    }

    let any_required_failed = results.values().any(|r| r.outcome == Outcome::Fail);

    if json_out {
        emit_json(&results, include_live);
    } else {
        emit_human(&results, include_live);
    }

    if any_required_failed {
        std::process::exit(1);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Pass,
    Fail,
    Missing,
}

impl Outcome {
    fn label(self) -> &'static str {
        match self {
            Outcome::Pass => "pass",
            Outcome::Fail => "fail",
            Outcome::Missing => "missing",
        }
    }
}

#[derive(Debug, Clone)]
struct FamilyResult {
    outcome: Outcome,
    detail: String,
    project_path: Option<PathBuf>,
    lockfile_path: Option<PathBuf>,
    environment_path: Option<PathBuf>,
}

impl FamilyResult {
    fn pass(detail: impl Into<String>) -> Self {
        FamilyResult {
            outcome: Outcome::Pass,
            detail: detail.into(),
            project_path: None,
            lockfile_path: None,
            environment_path: None,
        }
    }

    fn fail(detail: impl Into<String>) -> Self {
        FamilyResult {
            outcome: Outcome::Fail,
            detail: detail.into(),
            project_path: None,
            lockfile_path: None,
            environment_path: None,
        }
    }

    fn with_paths(
        mut self,
        project: Option<PathBuf>,
        lock: Option<PathBuf>,
        env: Option<PathBuf>,
    ) -> Self {
        self.project_path = project;
        self.lockfile_path = lock;
        self.environment_path = env;
        self
    }
}

fn run_family(family: &str, bin: &Path) -> FamilyResult {
    match family {
        "init" => probe_init(bin),
        "auth" => probe_auth(bin),
        "index" => probe_index(bin),
        "add" => probe_add(bin),
        "lock" => probe_lock(bin),
        "export" => probe_export(bin),
        "tree" => probe_tree(bin),
        "version" => probe_version(bin),
        "pip" => probe_pip(bin),
        "venv" => probe_venv(bin),
        "python" => probe_python(bin),
        "workspace" => probe_workspace(bin),
        "shell" => probe_shell(bin),
        "sync" => probe_sync(bin),
        "run" => probe_run(bin),
        "install" => probe_install(bin),
        "tool" => probe_tool(bin),
        "hash" => probe_hash(bin),
        "cache" => probe_cache(bin),
        other => FamilyResult {
            outcome: Outcome::Missing,
            detail: format!("no runner for family `{other}`"),
            project_path: None,
            lockfile_path: None,
            environment_path: None,
        },
    }
}

fn invoke(bin: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(bin)
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("spawn mamba")
}

fn build_frozen_index() -> ScratchDir {
    let dir = ScratchDir::new("index");
    stake_pkg(
        dir.path(),
        "frozen-demo-pkg",
        "0.1.0",
        &["frozen-demo-transitive==0.2.0"],
    );
    stake_pkg(dir.path(), "frozen-demo-transitive", "0.2.0", &[]);
    dir
}

fn setup_locked_project(bin: &Path, project: &Path, index: &Path) -> Option<FamilyResult> {
    if !invoke(bin, project, &["init"]).status.success() {
        return Some(FamilyResult::fail(
            "init failed before locked-project setup",
        ));
    }
    let add = invoke(
        bin,
        project,
        &[
            "add",
            "frozen-demo-pkg==0.1.0",
            "--index",
            index.to_str().unwrap(),
        ],
    );
    if !add.status.success() {
        return Some(FamilyResult::fail(format!(
            "add failed before locked-project setup: {}",
            String::from_utf8_lossy(&add.stderr)
        )));
    }
    let lock = invoke(bin, project, &["lock", "--index", index.to_str().unwrap()]);
    if !lock.status.success() {
        return Some(FamilyResult::fail(format!(
            "lock failed before locked-project setup: {}",
            String::from_utf8_lossy(&lock.stderr)
        )));
    }
    None
}

fn stake_pkg(index: &Path, normalized_name: &str, version: &str, requires: &[&str]) {
    let ver_dir = index.join(normalized_name).join(version);
    std::fs::create_dir_all(&ver_dir).unwrap();
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
}

fn probe_init(bin: &Path) -> FamilyResult {
    let tmp = ScratchDir::new("probe");
    let proj = tmp.path().to_path_buf();
    let out = invoke(bin, &proj, &["init"]);
    if !out.status.success() {
        return FamilyResult::fail(format!(
            "init exit {:?}: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    if !proj.join("mamba.toml").exists() {
        return FamilyResult::fail("init did not create mamba.toml");
    }
    FamilyResult::pass("init created mamba.toml + scaffolding").with_paths(Some(proj), None, None)
}

fn probe_auth(bin: &Path) -> FamilyResult {
    let tmp = ScratchDir::new("probe");
    let creds = tmp.path().join("credentials");
    let dir = Command::new(bin)
        .args(["auth", "dir"])
        .env("MAMBA_CREDENTIALS_DIR", &creds)
        .output()
        .expect("spawn mamba");
    if !dir.status.success()
        || String::from_utf8_lossy(&dir.stdout).trim_end() != creds.display().to_string()
    {
        return FamilyResult::fail(format!(
            "auth dir mismatch: {}",
            String::from_utf8_lossy(&dir.stdout)
        ));
    }

    let login = Command::new(bin)
        .args([
            "auth",
            "login",
            "https://Repo.EXAMPLE/simple",
            "--username",
            "alice",
            "--token",
            "secret-token",
        ])
        .env("MAMBA_CREDENTIALS_DIR", &creds)
        .output()
        .expect("spawn mamba");
    if !login.status.success() {
        return FamilyResult::fail(format!(
            "auth login failed: {}",
            String::from_utf8_lossy(&login.stderr)
        ));
    }

    let token = Command::new(bin)
        .args(["auth", "token", "repo.example", "--username", "alice"])
        .env("MAMBA_CREDENTIALS_DIR", &creds)
        .output()
        .expect("spawn mamba");
    if !token.status.success() || String::from_utf8_lossy(&token.stdout).trim() != "secret-token" {
        return FamilyResult::fail(format!(
            "auth token mismatch: {}",
            String::from_utf8_lossy(&token.stdout)
        ));
    }

    let logout = Command::new(bin)
        .args(["auth", "logout", "repo.example", "--username", "alice"])
        .env("MAMBA_CREDENTIALS_DIR", &creds)
        .output()
        .expect("spawn mamba");
    if !logout.status.success() {
        return FamilyResult::fail("auth logout failed");
    }

    FamilyResult::pass("auth dir/login/token/logout manage plaintext credentials").with_paths(
        None,
        None,
        Some(creds),
    )
}

fn probe_index(bin: &Path) -> FamilyResult {
    let tmp = ScratchDir::new("probe");
    let wheels = tmp.path().join("wheels");
    std::fs::create_dir(&wheels).unwrap();
    let filename = "frozen_demo_pkg-0.1.0-py3-none-any.whl";
    let wheel = wheels.join(filename);
    std::fs::write(&wheel, b"fake-wheel-bytes-for-index-build").unwrap();
    let index = tmp.path().join("index");

    let out = invoke(
        bin,
        tmp.path(),
        &[
            "index",
            "build",
            "--out",
            index.to_str().unwrap(),
            wheels.to_str().unwrap(),
        ],
    );
    if !out.status.success() {
        return FamilyResult::fail(format!(
            "index build exit {:?}: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        ));
    }

    let indexed = index.join("frozen-demo-pkg").join("0.1.0").join(filename);
    if !indexed.exists() {
        return FamilyResult::fail("index build did not materialize normalized package layout");
    }
    let first = std::fs::read(&indexed).unwrap();
    let replay = invoke(
        bin,
        tmp.path(),
        &[
            "index",
            "build",
            "--out",
            index.to_str().unwrap(),
            wheels.to_str().unwrap(),
        ],
    );
    if !replay.status.success() {
        return FamilyResult::fail("index build replay failed");
    }
    let second = std::fs::read(&indexed).unwrap();
    if first != second {
        return FamilyResult::fail("indexed wheel not byte-identical on replay");
    }

    FamilyResult::pass("index build materialized wheel layout; replay byte-identical").with_paths(
        Some(index),
        None,
        None,
    )
}

fn probe_add(bin: &Path) -> FamilyResult {
    let index = build_frozen_index();
    let tmp = ScratchDir::new("probe");
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    if !invoke(bin, &proj, &["init"]).status.success() {
        return FamilyResult::fail("init failed before add");
    }
    let out = invoke(
        bin,
        &proj,
        &[
            "add",
            "frozen-demo-pkg==0.1.0",
            "--index",
            index.path().to_str().unwrap(),
        ],
    );
    if !out.status.success() {
        return FamilyResult::fail(format!(
            "add exit {:?}: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let manifest = std::fs::read_to_string(proj.join("mamba.toml")).unwrap();
    if !manifest.contains("frozen-demo-pkg==0.1.0") {
        return FamilyResult::fail("manifest missing dep after add");
    }
    let lock_path = proj.join("mamba.lock");
    FamilyResult::pass("add recorded dep in manifest + lockfile").with_paths(
        Some(proj),
        Some(lock_path),
        None,
    )
}

fn probe_lock(bin: &Path) -> FamilyResult {
    let index = build_frozen_index();
    let tmp = ScratchDir::new("probe");
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    invoke(bin, &proj, &["init"]);
    invoke(
        bin,
        &proj,
        &[
            "add",
            "frozen-demo-pkg==0.1.0",
            "--index",
            index.path().to_str().unwrap(),
        ],
    );
    let lock_path = proj.join("mamba.lock");
    let out = invoke(
        bin,
        &proj,
        &["lock", "--index", index.path().to_str().unwrap()],
    );
    if !out.status.success() {
        return FamilyResult::fail(format!(
            "lock exit {:?}: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let a = std::fs::read(&lock_path).unwrap();
    invoke(
        bin,
        &proj,
        &["lock", "--index", index.path().to_str().unwrap()],
    );
    let b = std::fs::read(&lock_path).unwrap();
    if a != b {
        return FamilyResult::fail("lockfile not byte-identical on replay");
    }
    let lock = String::from_utf8_lossy(&a);
    if !lock.contains("name = \"frozen-demo-transitive\"") {
        return FamilyResult::fail("lockfile missing transitive dep");
    }
    FamilyResult::pass("lock byte-identical on replay; records transitive").with_paths(
        Some(proj),
        Some(lock_path),
        None,
    )
}

fn probe_export(bin: &Path) -> FamilyResult {
    let index = build_frozen_index();
    let tmp = ScratchDir::new("probe");
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    if let Some(failure) = setup_locked_project(bin, &proj, index.path()) {
        return failure;
    }
    let out = invoke(
        bin,
        &proj,
        &["export", "--no-header", "--no-hashes", "--annotate"],
    );
    if !out.status.success() {
        return FamilyResult::fail(format!(
            "export exit {:?}: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    if !stdout.contains("frozen-demo-pkg==0.1.0")
        || !stdout.contains("frozen-demo-transitive==0.2.0")
        || !stdout.contains("# via frozen-demo-pkg")
    {
        return FamilyResult::fail(format!(
            "requirements export missing pinned graph: {stdout}"
        ));
    }
    FamilyResult::pass("export emits requirements pins + reverse-dep annotations").with_paths(
        Some(proj.clone()),
        Some(proj.join("mamba.lock")),
        None,
    )
}

fn probe_tree(bin: &Path) -> FamilyResult {
    let index = build_frozen_index();
    let tmp = ScratchDir::new("probe");
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    if let Some(failure) = setup_locked_project(bin, &proj, index.path()) {
        return failure;
    }
    let out = invoke(bin, &proj, &["tree"]);
    if !out.status.success() {
        return FamilyResult::fail(format!(
            "tree exit {:?}: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    if !stdout.contains("frozen-demo-pkg v0.1.0")
        || !stdout.contains("frozen-demo-transitive v0.2.0")
    {
        return FamilyResult::fail(format!("tree missing locked graph: {stdout}"));
    }
    FamilyResult::pass("tree renders locked dependency graph").with_paths(
        Some(proj.clone()),
        Some(proj.join("mamba.lock")),
        None,
    )
}

fn probe_version(bin: &Path) -> FamilyResult {
    let tmp = ScratchDir::new("probe");
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    let pyproject = proj.join("pyproject.toml");
    std::fs::write(
        &pyproject,
        "[project]\nname = \"demo\"\nversion = \"1.2.3\"\n",
    )
    .unwrap();
    let out = invoke(bin, &proj, &["version", "--bump", "patch"]);
    if !out.status.success() {
        return FamilyResult::fail(format!(
            "version exit {:?}: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let body = std::fs::read_to_string(&pyproject).unwrap_or_default();
    if stdout.trim() != "1.2.4" || !body.contains("version = \"1.2.4\"") {
        return FamilyResult::fail(format!("version did not bump pyproject: {stdout} / {body}"));
    }
    FamilyResult::pass("version bumps PEP 621 project version").with_paths(Some(proj), None, None)
}

fn probe_pip(bin: &Path) -> FamilyResult {
    let tmp = ScratchDir::new("probe");
    let site = tmp.path().join("site-packages");
    std::fs::create_dir_all(&site).unwrap();
    write_dist(
        &site,
        "Requests-2.31.0.dist-info",
        "Name: Requests\nVersion: 2.31.0\nRequires-Dist: urllib3>=2\n",
    );
    write_dist(
        &site,
        "urllib3-2.1.0.dist-info",
        "Name: urllib3\nVersion: 2.1.0\n",
    );

    let check = invoke(
        bin,
        tmp.path(),
        &["pip", "check", "--site-packages", site.to_str().unwrap()],
    );
    if !check.status.success() {
        return FamilyResult::fail(format!(
            "pip check exit {:?}: {}",
            check.status.code(),
            String::from_utf8_lossy(&check.stdout)
        ));
    }
    let freeze = invoke(
        bin,
        tmp.path(),
        &["pip", "freeze", "--site-packages", site.to_str().unwrap()],
    );
    if !freeze.status.success() {
        return FamilyResult::fail("pip freeze failed");
    }
    let stdout = String::from_utf8_lossy(&freeze.stdout);
    if !stdout.contains("Requests==2.31.0") || !stdout.contains("urllib3==2.1.0") {
        return FamilyResult::fail(format!("pip freeze missing inventory: {stdout}"));
    }
    FamilyResult::pass("pip list/freeze/check inspect site-packages inventory").with_paths(
        Some(tmp.path().to_path_buf()),
        None,
        Some(site),
    )
}

fn write_dist(site: &Path, dir_name: &str, metadata: &str) {
    let dist = site.join(dir_name);
    std::fs::create_dir_all(&dist).unwrap();
    std::fs::write(dist.join("METADATA"), metadata).unwrap();
}

fn probe_venv(bin: &Path) -> FamilyResult {
    let tmp = ScratchDir::new("probe");
    let root = tmp.path().join("v");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("pyvenv.cfg"),
        "home = /tmp\ninclude-system-site-packages = false\nversion = 3.12.0\n",
    )
    .unwrap();

    let refuse = invoke(
        bin,
        tmp.path(),
        &[
            "venv",
            "create",
            root.to_str().unwrap(),
            "--python",
            "/definitely/missing/python",
        ],
    );
    if refuse.status.success() {
        return FamilyResult::fail("venv create overwrote existing pyvenv.cfg");
    }
    let stderr = String::from_utf8_lossy(&refuse.stderr);
    if !stderr.contains("refused_existing_pyvenv_cfg") {
        return FamilyResult::fail(format!("venv create refusal missing reason: {stderr}"));
    }

    let remove = invoke(bin, tmp.path(), &["venv", "remove", root.to_str().unwrap()]);
    if !remove.status.success() {
        return FamilyResult::fail(format!(
            "venv remove exit {:?}: {}",
            remove.status.code(),
            String::from_utf8_lossy(&remove.stderr)
        ));
    }
    if root.exists() {
        return FamilyResult::fail("venv remove left target tree behind");
    }
    FamilyResult::pass("venv create refuses overwrite and remove requires pyvenv.cfg").with_paths(
        Some(root),
        None,
        None,
    )
}

fn probe_python(bin: &Path) -> FamilyResult {
    let tmp = ScratchDir::new("probe");
    let project = tmp.path().join("project");
    let data = tmp.path().join("uv-data");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::create_dir_all(tmp.path().join("empty-path")).unwrap();

    let pin = invoke(bin, &project, &["python", "pin", "3.12"]);
    if !pin.status.success() {
        return FamilyResult::fail(format!(
            "python pin exit {:?}: {}",
            pin.status.code(),
            String::from_utf8_lossy(&pin.stderr)
        ));
    }
    let pin_body = std::fs::read_to_string(project.join(".python-version")).unwrap_or_default();
    if pin_body != "3.12\n" {
        return FamilyResult::fail(format!("python pin body mismatch: {pin_body:?}"));
    }

    let dir = Command::new(bin)
        .args(["python", "dir"])
        .env("UV_DATA_DIR", &data)
        .output()
        .expect("spawn mamba");
    if !dir.status.success() {
        return FamilyResult::fail("python dir failed");
    }
    let printed = String::from_utf8_lossy(&dir.stdout).trim_end().to_string();
    if printed != data.join("python").to_string_lossy() {
        return FamilyResult::fail(format!("python dir mismatch: {printed}"));
    }

    let list = Command::new(bin)
        .args(["python", "list"])
        .env("PATH", tmp.path().join("empty-path"))
        .output()
        .expect("spawn mamba");
    if !list.status.success() {
        return FamilyResult::fail("python list failed with empty PATH");
    }
    FamilyResult::pass("python list/pin/dir expose local interpreter management").with_paths(
        Some(project),
        None,
        Some(data),
    )
}

fn probe_workspace(bin: &Path) -> FamilyResult {
    let tmp = ScratchDir::new("probe");
    let root = tmp.path();
    std::fs::write(
        root.join("pyproject.toml"),
        r#"
[tool.uv.workspace]
members = ["packages/*"]
exclude = ["packages/skip"]
"#,
    )
    .unwrap();
    write_workspace_member(root, "packages/alpha", "Alpha_Pkg", "0.1.0");
    write_workspace_member(root, "packages/skip", "skip", "9.9.9");

    let out = invoke(bin, root, &["workspace", "list"]);
    if !out.status.success() {
        return FamilyResult::fail(format!(
            "workspace list exit {:?}: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    if stdout.trim() != "alpha-pkg" || stdout.contains("skip") {
        return FamilyResult::fail(format!("workspace list output mismatch: {stdout}"));
    }

    let paths = invoke(bin, root, &["workspace", "list", "--paths"]);
    if !paths.status.success()
        || !String::from_utf8_lossy(&paths.stdout)
            .trim_end()
            .ends_with("packages/alpha")
    {
        return FamilyResult::fail(format!(
            "workspace list --paths mismatch: {} / {}",
            paths.status,
            String::from_utf8_lossy(&paths.stdout)
        ));
    }

    let dir = invoke(bin, root, &["workspace", "dir", "--package", "alpha_pkg"]);
    if !dir.status.success()
        || !String::from_utf8_lossy(&dir.stdout)
            .trim_end()
            .ends_with("packages/alpha")
    {
        return FamilyResult::fail(format!(
            "workspace dir --package mismatch: {} / {}",
            dir.status,
            String::from_utf8_lossy(&dir.stdout)
        ));
    }

    let metadata = invoke(bin, root, &["workspace", "metadata"]);
    let metadata_stdout = String::from_utf8_lossy(&metadata.stdout);
    if !metadata.status.success()
        || !metadata_stdout.contains("\"workspace\"")
        || !metadata_stdout.contains("\"members\"")
        || !metadata_stdout.contains("\"exclude\"")
        || metadata_stdout.contains("\"name\":\"skip\"")
    {
        return FamilyResult::fail(format!("workspace metadata mismatch: {metadata_stdout}"));
    }

    FamilyResult::pass("workspace list/dir/metadata inspect uv workspace members").with_paths(
        Some(root.to_path_buf()),
        None,
        None,
    )
}

fn write_workspace_member(root: &Path, rel: &str, name: &str, version: &str) {
    let dir = root.join(rel);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("pyproject.toml"),
        format!("[project]\nname = {name:?}\nversion = {version:?}\n"),
    )
    .unwrap();
}

fn probe_shell(bin: &Path) -> FamilyResult {
    let tmp = ScratchDir::new("probe");
    let path = invoke(
        bin,
        tmp.path(),
        &[
            "shell",
            "path",
            "--shell",
            "bash",
            "--bin-dir",
            "/opt/mamba/bin",
        ],
    );
    let path_stdout = String::from_utf8_lossy(&path.stdout);
    if !path.status.success() || path_stdout.trim() != r#"export PATH="/opt/mamba/bin:$PATH""# {
        return FamilyResult::fail(format!("shell path mismatch: {path_stdout}"));
    }

    let init = invoke(
        bin,
        tmp.path(),
        &[
            "shell",
            "init",
            "--shell",
            "nushell",
            "--bin-dir",
            "/opt/mamba/bin",
        ],
    );
    let init_stdout = String::from_utf8_lossy(&init.stdout);
    if !init.status.success()
        || !init_stdout.contains("# >>> mamba initialize >>>")
        || !init_stdout.contains("$env.PATH = ($env.PATH | prepend \"/opt/mamba/bin\")")
        || !init_stdout.contains("# <<< mamba initialize <<<")
    {
        return FamilyResult::fail(format!("shell init mismatch: {init_stdout}"));
    }

    let completion = invoke(bin, tmp.path(), &["generate-shell-completion", "bash"]);
    let completion_stdout = String::from_utf8_lossy(&completion.stdout);
    if !completion.status.success()
        || !completion_stdout.contains("workspace")
        || !completion_stdout.contains("pkgmgr-validate")
        || !completion_stdout.contains("generate-shell-completion")
    {
        return FamilyResult::fail("shell completion missing expected command tree");
    }

    FamilyResult::pass("shell path/init and completion generation are wired")
}

fn probe_sync(bin: &Path) -> FamilyResult {
    let index = build_frozen_index();
    let tmp = ScratchDir::new("probe");
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    invoke(bin, &proj, &["init"]);
    invoke(
        bin,
        &proj,
        &[
            "add",
            "frozen-demo-pkg==0.1.0",
            "--index",
            index.path().to_str().unwrap(),
        ],
    );
    invoke(
        bin,
        &proj,
        &["lock", "--index", index.path().to_str().unwrap()],
    );
    let first = invoke(bin, &proj, &["sync"]);
    if !first.status.success() {
        return FamilyResult::fail("first sync failed");
    }
    let second = invoke(bin, &proj, &["sync"]);
    if !second.status.success() {
        return FamilyResult::fail("second sync failed");
    }
    let stderr = String::from_utf8_lossy(&second.stderr);
    if !stderr.contains("no_op") {
        return FamilyResult::fail("second sync did not signal no_op");
    }
    FamilyResult::pass("sync first-run installs; second-run no_op").with_paths(
        Some(proj.clone()),
        Some(proj.join("mamba.lock")),
        Some(proj.join(".venv")),
    )
}

fn probe_run(bin: &Path) -> FamilyResult {
    let index = build_frozen_index();
    let tmp = ScratchDir::new("probe");
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    invoke(bin, &proj, &["init"]);
    invoke(
        bin,
        &proj,
        &[
            "add",
            "frozen-demo-pkg==0.1.0",
            "--index",
            index.path().to_str().unwrap(),
        ],
    );
    invoke(
        bin,
        &proj,
        &["lock", "--index", index.path().to_str().unwrap()],
    );
    std::fs::create_dir_all(proj.join("scripts")).unwrap();
    std::fs::write(proj.join("scripts/hello.py"), "print('OK')\n").unwrap();
    let out = invoke(bin, &proj, &["run", "scripts/hello.py"]);
    if out.status.success() {
        return FamilyResult::fail("run before sync must fail (env not synced)");
    }
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !stderr.contains("environment is not synced") {
        return FamilyResult::fail("run pre-sync did not name env-not-synced");
    }
    FamilyResult::pass("run preflight rejects un-synced env").with_paths(
        Some(proj.clone()),
        Some(proj.join("mamba.lock")),
        Some(proj.join(".venv")),
    )
}

fn probe_install(bin: &Path) -> FamilyResult {
    let index = build_frozen_index();
    let tmp = ScratchDir::new("probe");
    let tools = tmp.path().join("mamba-tools");
    let out = Command::new(bin)
        .args([
            "install",
            "frozen-demo-pkg",
            "--version",
            "0.1.0",
            "--index",
            index.path().to_str().unwrap(),
        ])
        .env("MAMBA_TOOLS_DIR", &tools)
        .output()
        .expect("spawn mamba");
    if !out.status.success() {
        return FamilyResult::fail(format!(
            "install exit {:?}: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let tool_dir = tools.join("frozen-demo-pkg");
    if !tool_dir.join("manifest.toml").exists() {
        return FamilyResult::fail("install did not materialize manifest");
    }
    FamilyResult::pass("install materialized tool dir + shim").with_paths(
        Some(tool_dir),
        None,
        Some(tools),
    )
}

fn probe_tool(bin: &Path) -> FamilyResult {
    let index = build_frozen_index();
    stake_pkg(index.path(), "frozen-demo-pkg", "0.2.0", &[]);
    let tmp = ScratchDir::new("probe");
    let tools = tmp.path().join("mamba-tools");

    let dir = Command::new(bin)
        .args(["tool", "dir"])
        .env("MAMBA_TOOLS_DIR", &tools)
        .output()
        .expect("spawn mamba");
    if !dir.status.success()
        || String::from_utf8_lossy(&dir.stdout).trim_end() != tools.display().to_string()
    {
        return FamilyResult::fail("tool dir did not print tools root");
    }

    let install = Command::new(bin)
        .args([
            "tool",
            "install",
            "frozen-demo-pkg",
            "--version",
            "0.1.0",
            "--index",
            index.path().to_str().unwrap(),
        ])
        .env("MAMBA_TOOLS_DIR", &tools)
        .output()
        .expect("spawn mamba");
    if !install.status.success() {
        return FamilyResult::fail(format!(
            "tool install failed: {}",
            String::from_utf8_lossy(&install.stderr)
        ));
    }

    let upgrade = Command::new(bin)
        .args([
            "tool",
            "upgrade",
            "frozen-demo-pkg",
            "--index",
            index.path().to_str().unwrap(),
        ])
        .env("MAMBA_TOOLS_DIR", &tools)
        .output()
        .expect("spawn mamba");
    if !upgrade.status.success() {
        return FamilyResult::fail("tool upgrade failed");
    }
    let manifest =
        std::fs::read_to_string(tools.join("frozen-demo-pkg/manifest.toml")).unwrap_or_default();
    if !manifest.contains("version = \"0.2.0\"") {
        return FamilyResult::fail(format!("tool upgrade did not install latest: {manifest}"));
    }

    let list = Command::new(bin)
        .args(["tool", "list"])
        .env("MAMBA_TOOLS_DIR", &tools)
        .output()
        .expect("spawn mamba");
    if !list.status.success()
        || !String::from_utf8_lossy(&list.stdout).contains("frozen-demo-pkg==0.2.0")
    {
        return FamilyResult::fail("tool list did not show upgraded tool");
    }

    let shell = Command::new(bin)
        .args([
            "tool",
            "update-shell",
            "--shell",
            "bash",
            "--bin-dir",
            "/opt/mamba/bin",
        ])
        .env("MAMBA_TOOLS_DIR", &tools)
        .output()
        .expect("spawn mamba");
    if !shell.status.success()
        || !String::from_utf8_lossy(&shell.stdout).contains("export PATH=\"/opt/mamba/bin:$PATH\"")
    {
        return FamilyResult::fail("tool update-shell did not emit PATH snippet");
    }

    let uninstall = Command::new(bin)
        .args(["tool", "uninstall", "frozen-demo-pkg"])
        .env("MAMBA_TOOLS_DIR", &tools)
        .output()
        .expect("spawn mamba");
    if !uninstall.status.success() || tools.join("frozen-demo-pkg").exists() {
        return FamilyResult::fail("tool uninstall failed");
    }

    FamilyResult::pass("tool install/upgrade/list/dir/update-shell/uninstall are wired").with_paths(
        None,
        None,
        Some(tools),
    )
}

fn probe_hash(bin: &Path) -> FamilyResult {
    let tmp = ScratchDir::new("probe");
    let blob = tmp.path().join("hello.txt");
    std::fs::write(&blob, b"hello\n").unwrap();
    let out = invoke(bin, tmp.path(), &["hash", blob.to_str().unwrap()]);
    if !out.status.success() {
        return FamilyResult::fail("hash failed");
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let expected_prefix =
        "sha256:5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03  ";
    if !stdout.starts_with(expected_prefix) {
        return FamilyResult::fail(format!("hash baseline mismatch: {stdout}"));
    }
    FamilyResult::pass("hash matches pinned baseline").with_paths(None, None, None)
}

fn probe_cache(bin: &Path) -> FamilyResult {
    let tmp = ScratchDir::new("probe");
    let cache_root = tmp.path().join("mamba-cache");
    std::fs::create_dir_all(&cache_root).unwrap();
    std::fs::create_dir_all(cache_root.join("artifacts/demo")).unwrap();
    std::fs::write(cache_root.join("artifacts/demo/blob.whl"), b"xyz").unwrap();

    let dir_out = Command::new(bin)
        .args(["cache", "dir"])
        .env("MAMBA_CACHE_DIR", &cache_root)
        .env_remove("XDG_CACHE_HOME")
        .output()
        .expect("spawn mamba");
    if !dir_out.status.success() {
        return FamilyResult::fail("cache dir failed");
    }
    let printed = String::from_utf8_lossy(&dir_out.stdout)
        .trim_end()
        .to_string();
    if printed != cache_root.to_string_lossy() {
        return FamilyResult::fail(format!(
            "cache dir output mismatch: {} != {}",
            printed,
            cache_root.display()
        ));
    }

    let size_out = Command::new(bin)
        .args(["cache", "size"])
        .env("MAMBA_CACHE_DIR", &cache_root)
        .output()
        .expect("spawn mamba");
    if !size_out.status.success() {
        return FamilyResult::fail("cache size failed");
    }
    let size_stdout = String::from_utf8_lossy(&size_out.stdout);
    if !size_stdout.contains("3 bytes") {
        return FamilyResult::fail(format!("cache size output mismatch: {size_stdout}"));
    }

    let clean_out = Command::new(bin)
        .args(["cache", "clean"])
        .env("MAMBA_CACHE_DIR", &cache_root)
        .output()
        .expect("spawn mamba");
    if !clean_out.status.success() {
        return FamilyResult::fail("cache clean failed");
    }
    if cache_root.join("artifacts/demo/blob.whl").exists() {
        return FamilyResult::fail("cache clean did not remove blob");
    }
    FamilyResult::pass("cache dir/size/clean honor MAMBA_CACHE_DIR").with_paths(
        None,
        None,
        Some(cache_root),
    )
}

fn emit_human(results: &BTreeMap<String, FamilyResult>, include_live: bool) {
    eprintln!("mamba pkgmgr-validate — release-blocking workflows");
    eprintln!("  network: offline   live: {}", include_live);
    eprintln!();
    let mut passed = 0usize;
    let mut failed = 0usize;
    for fam in REQUIRED_FAMILIES {
        let r = results.get(*fam).unwrap();
        if r.outcome == Outcome::Pass {
            passed += 1;
        } else {
            failed += 1;
        }
        eprintln!(
            "  [{:>4}] {fam:<8} {} {}",
            r.outcome.label(),
            r.detail,
            r.project_path
                .as_ref()
                .map(|p| format!("project={}", p.display()))
                .unwrap_or_default(),
        );
    }
    eprintln!();
    eprintln!("summary: {passed} passed, {failed} failed");
}

fn emit_json(results: &BTreeMap<String, FamilyResult>, include_live: bool) {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!("  \"include_live_network\": {include_live},\n"));
    out.push_str("  \"families\": {\n");
    let n = REQUIRED_FAMILIES.len();
    for (i, fam) in REQUIRED_FAMILIES.iter().enumerate() {
        let r = results.get(*fam).unwrap();
        let passed = if r.outcome == Outcome::Pass { 1 } else { 0 };
        let failed = if r.outcome == Outcome::Fail { 1 } else { 0 };
        let missing = if r.outcome == Outcome::Missing { 1 } else { 0 };
        out.push_str(&format!("    \"{fam}\": {{\n"));
        out.push_str(&format!("      \"outcome\": \"{}\",\n", r.outcome.label()));
        out.push_str(&format!("      \"passed\": {passed},\n"));
        out.push_str(&format!("      \"failed\": {failed},\n"));
        out.push_str(&format!("      \"missing\": {missing},\n"));
        out.push_str(&format!("      \"fixtures\": 1,\n"));
        out.push_str(&format!("      \"detail\": {},\n", json_str(&r.detail)));
        out.push_str(&format!(
            "      \"project_path\": {},\n",
            json_opt_path(&r.project_path)
        ));
        out.push_str(&format!(
            "      \"lockfile_path\": {},\n",
            json_opt_path(&r.lockfile_path)
        ));
        out.push_str(&format!(
            "      \"environment_path\": {}\n",
            json_opt_path(&r.environment_path)
        ));
        out.push_str(if i + 1 == n { "    }\n" } else { "    },\n" });
    }
    out.push_str("  }\n");
    out.push_str("}\n");
    print!("{out}");
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_opt_path(p: &Option<PathBuf>) -> String {
    match p {
        Some(path) => json_str(&path.to_string_lossy()),
        None => "null".to_string(),
    }
}
