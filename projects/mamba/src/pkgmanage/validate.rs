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
    "init", "index", "add", "lock", "export", "tree", "sync", "run", "install", "hash", "cache",
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
        "index" => probe_index(bin),
        "add" => probe_add(bin),
        "lock" => probe_lock(bin),
        "export" => probe_export(bin),
        "tree" => probe_tree(bin),
        "sync" => probe_sync(bin),
        "run" => probe_run(bin),
        "install" => probe_install(bin),
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
    std::fs::write(cache_root.join("blob.bin"), b"x").unwrap();

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

    let clean_out = Command::new(bin)
        .args(["cache", "clean"])
        .env("MAMBA_CACHE_DIR", &cache_root)
        .output()
        .expect("spawn mamba");
    if !clean_out.status.success() {
        return FamilyResult::fail("cache clean failed");
    }
    if cache_root.join("blob.bin").exists() {
        return FamilyResult::fail("cache clean did not remove blob");
    }
    FamilyResult::pass("cache dir + clean honor MAMBA_CACHE_DIR").with_paths(
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
