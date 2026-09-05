//! Black-box contract: `mamba pkgmgr-validate` must report its `auth` family
//! as **passing**, because the authenticated `add` that family performs
//! against its own in-process mock registry locks the requested package under
//! the PEP 503 canonical name the resolver produces.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary and reads only what the binary
//! wrote to its own stdout and stderr. It links no part of the validator, the
//! resolver, or the lock renderer; it starts no registry, writes no
//! `mamba.lock`, and hand-builds no fixture. Every byte the `auth` probe sees
//! comes from the mock registry the product spawns inside itself, so a green
//! run means the product's authenticated add-and-lock path agreed with the
//! product's own verdict on it — not that the case agreed with itself.
//!
//! # The observation point
//!
//! Two runs of the binary, each from its own freshly created temporary
//! directory that holds no `mamba.toml` (and has no `mamba.toml` in any
//! ancestor), with `HOME`, the mamba cache directory, and the mamba credential
//! directory pointed at temporary paths of their own:
//!
//! | run | observed |
//! |---|---|
//! | `mamba pkgmgr-validate --json` | exit status, and the `auth` object inside `"families"` |
//! | `mamba pkgmgr-validate` | exit status, the `[pass] auth` line, and the summary count |
//!
//! ## Behaviour
//!
//! The `auth` family object must report `"outcome": "pass"` with
//! `"failed": 0`, and the command must exit `0`. `outcome` is asserted
//! alongside `failed` on purpose: a family that stopped running would report
//! `"missing"` with `"failed": 0` too, and that is not a pass.
//!
//! `pkgmgr-validate` has no family filter, so the exit status is the verdict
//! over the whole required set. The human run therefore also pins the summary
//! literal `summary: 21 passed, 0 failed` and the JSON run pins that the
//! document still describes exactly 21 families — dropping, renaming, or
//! short-circuiting a family to reach exit `0` moves one of those two numbers.
//!
//! ## Security
//!
//! `--include-live-network` is never passed, and the case asserts the
//! validator reported `"include_live_network": false` back: the network-free
//! path is the default, and the run that produced this verdict took it. The
//! environment removes `MAMBA_FROZEN_INDEX`, `MAMBA_INDEX_URL`,
//! `XDG_CACHE_HOME`, `MAMBA_JOBS`, `VIRTUAL_ENV`, `PYTHONPATH`, and every
//! proxy variable, so no ambient index, cache, or proxy can supply or divert
//! an answer; the only registry in reach is the loopback mock the `auth` probe
//! starts for itself.
//!
//! That the authenticated path was *exercised* is what a passing `auth` family
//! means, and nothing weaker: the probe fails the family with a distinct
//! detail at `auth dir`, `auth login`, `auth token`, `auth logout`, the index
//! login, and the authenticated `add` itself, so `"outcome": "pass"` is
//! reachable only by walking the whole credential-store-then-authenticated-add
//! chain and having the resulting lock accepted.
//!
//! The case also asserts the validator's own output carries neither token the
//! probe stores while doing it. This is an absence assertion over the observed
//! surface — it can never turn red for a renamed literal, only for output that
//! started echoing credential material into a failure detail.
//!
//! ## Performance
//!
//! None. This work item names no performance budget, so the case asserts no
//! duration. The deadline below is a hang detector, not a budget: it exists so
//! a wedged probe produces a readable red naming the run and its partial
//! output instead of a test binary that never returns. It is set far above the
//! whole-run cost so that it can never be the thing that fails.
//!
//! # Why today's tree cannot pass
//!
//! The `auth` probe runs `add auth_demo --index-url <mock>` and then requires
//! the lock text to contain the literal `name = "auth_demo"`. Since registry
//! adds began rendering through the resolver, the pin the resolver writes
//! carries the PEP 503 canonical name — `name = "auth-demo"` — so the literal
//! is never found. Today the family reports `"outcome": "fail"` with
//! `"failed": 1` and the detail `authenticated add did not lock auth_demo`,
//! the human run prints `[fail] auth` and `summary: 20 passed, 1 failed`, and
//! the command exits `1`.
//!
//! # Reading the JSON
//!
//! `serde_json` is a dependency of the `mamba` library, not a dev-dependency
//! of this crate, so an integration test cannot link it — and adding a crate
//! is outside this phase's write boundary. The reader below is therefore the
//! case's own, and it is exact rather than approximate because of a property
//! of the emitter it reads: the validator writes one field per physical line
//! and escapes every control character inside a value, so no value can carry a
//! raw newline and no field line can be mistaken for a structural one. Values
//! are compared as the raw tokens they are printed as — `"pass"` with its
//! quotes, `0` without — so a string can never be confused with a number.
//!
//! Nothing here is `#[ignore]`d, nothing is skipped, and nothing retries. A
//! host without `python3` on `PATH` fails this case naming it, because the
//! validator's environment-building families need an interpreter and a case
//! that quietly skipped would report a verdict it never measured.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// The family this work item is about.
const FAMILY: &str = "auth";

/// The size of the required family set. `pkgmgr-validate` exits `0` only when
/// every one of them passes, so this count and the summary literal below are
/// what stop a dropped family from buying a green.
const REQUIRED_FAMILY_COUNT: usize = 21;

/// The human summary that corresponds to a fully passing run.
const EXPECTED_SUMMARY: &str = "summary: 21 passed, 0 failed";

/// The human verdict line for the family under contract.
const EXPECTED_HUMAN_LINE: &str = "[pass] auth";

/// Credential material the `auth` probe stores and sends while proving the
/// authenticated path. None of it belongs in the validator's own output.
const PROBE_CREDENTIALS: [&str; 2] = ["secret-token", "index-token"];

/// A hang detector, not a performance budget — see the module docs. The whole
/// command is 21 families of subprocess work; this is set far above its cost
/// so that only a wedged run can reach it.
const HANG_DEADLINE: Duration = Duration::from_secs(600);

/// How often the watchdog asks whether the child has exited.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// The binary under test. `CARGO_BIN_EXE_*` is resolved by cargo at compile
/// time, so a missing binary is a build failure rather than a skipped case.
fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

/// One completed run of the binary, with everything a reader needs to act on a
/// red: the invocation, the status, and both streams.
struct Run {
    args: Vec<String>,
    cwd: PathBuf,
    code: Option<i32>,
    stdout: String,
    stderr: String,
    elapsed: Duration,
}

impl Run {
    fn render(&self) -> String {
        format!(
            "`mamba {}` in {} exited {:?} after {:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
            self.args.join(" "),
            self.cwd.display(),
            self.code,
            self.elapsed,
            self.stdout,
            self.stderr,
        )
    }
}

/// The throwaway tree both runs live in. Each run gets its own `cwd`, `HOME`,
/// cache, and credential directory underneath it, so neither run can observe
/// anything the other left behind.
struct Sandbox {
    root: tempfile::TempDir,
}

impl Sandbox {
    fn new() -> Self {
        Sandbox {
            root: tempfile::tempdir().expect("fixture: create the sandbox root"),
        }
    }

    /// Run the binary once, from a directory created for this run alone.
    fn run(&self, tag: &str, args: &[&str]) -> Run {
        let base = self.root.path().join(tag);
        let cwd = base.join("cwd");
        let home = base.join("home");
        let cache = base.join("cache");
        let creds = base.join("credentials");
        let logs = base.join("logs");
        for dir in [&cwd, &home, &cache, &creds, &logs] {
            std::fs::create_dir_all(dir)
                .unwrap_or_else(|e| panic!("fixture: create {}: {e}", dir.display()));
        }
        assert_no_project_manifest(&cwd);

        let out_path = logs.join("stdout");
        let err_path = logs.join("stderr");
        let out_file = File::create(&out_path)
            .unwrap_or_else(|e| panic!("fixture: create {}: {e}", out_path.display()));
        let err_file = File::create(&err_path)
            .unwrap_or_else(|e| panic!("fixture: create {}: {e}", err_path.display()));

        let mut child = Command::new(mamba_bin())
            .args(args)
            .current_dir(&cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::from(out_file))
            .stderr(Stdio::from(err_file))
            .env("HOME", &home)
            .env("MAMBA_CACHE_DIR", &cache)
            .env("MAMBA_CREDENTIALS_DIR", &creds)
            .env_remove("MAMBA_FROZEN_INDEX")
            .env_remove("MAMBA_INDEX_URL")
            .env_remove("MAMBA_JOBS")
            .env_remove("XDG_CACHE_HOME")
            .env_remove("VIRTUAL_ENV")
            .env_remove("PYTHONPATH")
            .env_remove("HTTP_PROXY")
            .env_remove("HTTPS_PROXY")
            .env_remove("ALL_PROXY")
            .env_remove("http_proxy")
            .env_remove("https_proxy")
            .env_remove("all_proxy")
            .spawn()
            .unwrap_or_else(|e| panic!("spawn {} {args:?}: {e}", mamba_bin().display()));

        let started = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {}
                Err(e) => panic!("wait on `mamba {}`: {e}", args.join(" ")),
            }
            if started.elapsed() >= HANG_DEADLINE {
                let _ = child.kill();
                let _ = child.wait();
                panic!(
                    "`mamba {}` did not exit within {HANG_DEADLINE:?} and was killed. This is a \
                     hang, not a slow run: the whole command is normally seconds of work. \
                     Partial output follows.\n--- stdout so far ---\n{}\n--- stderr so far ---\n{}",
                    args.join(" "),
                    read_lossy(&out_path),
                    read_lossy(&err_path),
                );
            }
            std::thread::sleep(POLL_INTERVAL);
        };

        Run {
            args: args.iter().map(|a| (*a).to_string()).collect(),
            cwd,
            code: status.code(),
            stdout: read_lossy(&out_path),
            stderr: read_lossy(&err_path),
            elapsed: started.elapsed(),
        }
    }
}

fn read_lossy(path: &Path) -> String {
    let bytes =
        std::fs::read(path).unwrap_or_else(|e| panic!("read back {}: {e}", path.display()));
    String::from_utf8_lossy(&bytes).to_string()
}

/// The working directory must not sit inside a mamba project: the validator's
/// families build their own projects in scratch directories, and an ancestor
/// manifest would let an unrelated tree answer for them.
fn assert_no_project_manifest(cwd: &Path) {
    for ancestor in cwd.ancestors() {
        let manifest = ancestor.join("mamba.toml");
        assert!(
            !manifest.exists(),
            "this case runs `mamba pkgmgr-validate` from a directory with no project above it, \
             and found {}",
            manifest.display()
        );
    }
}

/// The `python3` the validator's environment-building families need. It is a
/// requirement of this case, not an option: a host without one fails here,
/// naming it, and the case never skips.
fn require_python3() {
    let path_var = std::env::var_os("PATH")
        .unwrap_or_else(|| panic!("this case needs `python3`, and PATH is unset"));
    let mut candidate: Option<PathBuf> = None;
    for dir in std::env::split_paths(&path_var) {
        for name in ["python3", "python3.exe"] {
            let probe = dir.join(name);
            if probe.is_file() {
                candidate = Some(probe);
                break;
            }
        }
        if candidate.is_some() {
            break;
        }
    }
    let candidate = candidate.unwrap_or_else(|| {
        panic!(
            "this case needs `python3` on PATH — `mamba pkgmgr-validate` builds project \
             environments from it — and found none in {path_var:?}"
        )
    });
    let out = Command::new(&candidate)
        .args(["-c", "import sys; print(sys.executable)"])
        .output()
        .unwrap_or_else(|e| panic!("spawn {}: {e}", candidate.display()));
    assert!(
        out.status.success(),
        "`{} -c 'print(sys.executable)'` failed, so the validator has no interpreter to build \
         project environments from: {}",
        candidate.display(),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Every family key the JSON document declares, in the order it declares them.
///
/// A family object opens the only lines inside `"families"` that end in `: {`;
/// every field line ends in a scalar. See the module docs for why a
/// line-oriented reader is exact here.
fn family_names(json: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut inside = false;
    for line in json.lines() {
        let trimmed = line.trim();
        if trimmed == "\"families\": {" {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if let Some(rest) = trimmed.strip_suffix(": {") {
            if let Some(name) = rest.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                names.push(name.to_string());
            }
        }
    }
    names
}

/// The lines of one family's object, without its braces.
fn family_object<'a>(json: &'a str, family: &str, run: &Run) -> Vec<&'a str> {
    let opener = format!("\"{family}\": {{");
    let mut lines = json.lines();
    let found = lines.any(|line| line.trim() == opener);
    assert!(
        found,
        "the `{family}` family is missing from the JSON `pkgmgr-validate` printed; it declared \
         {:?}\n{}",
        family_names(json),
        run.render()
    );
    let mut block = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "}" || trimmed == "}," {
            return block;
        }
        block.push(line);
    }
    panic!(
        "the `{family}` family object in the JSON `pkgmgr-validate` printed is never closed\n{}",
        run.render()
    );
}

/// One field of a family object, as the raw token it was printed as: a string
/// keeps its quotes, a number does not, so the two can never be confused.
fn field(block: &[&str], key: &str, family: &str, run: &Run) -> String {
    let prefix = format!("\"{key}\":");
    let mut found: Option<String> = None;
    for line in block {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix(&prefix) else {
            continue;
        };
        let value = rest.trim().trim_end_matches(',').trim().to_string();
        assert!(
            found.is_none(),
            "the `{family}` family object declares `{key}` more than once\n{}",
            run.render()
        );
        found = Some(value);
    }
    found.unwrap_or_else(|| {
        panic!(
            "the `{family}` family object has no `{key}` field; it declared:\n{}\n{}",
            block.join("\n"),
            run.render()
        )
    })
}

/// A field that appears exactly once in the whole document.
fn unique_field(json: &str, key: &str, run: &Run) -> String {
    let prefix = format!("\"{key}\":");
    let values: Vec<String> = json
        .lines()
        .filter_map(|line| line.trim().strip_prefix(&prefix).map(str::to_string))
        .map(|rest| rest.trim().trim_end_matches(',').trim().to_string())
        .collect();
    assert_eq!(
        values.len(),
        1,
        "expected exactly one `{key}` in the JSON `pkgmgr-validate` printed, found {}\n{}",
        values.len(),
        run.render()
    );
    values.into_iter().next().expect("checked above")
}

/// The validator must not echo the credentials its own `auth` probe uses.
fn assert_no_credential_leak(run: &Run) {
    for secret in PROBE_CREDENTIALS {
        for (stream, body) in [("stdout", &run.stdout), ("stderr", &run.stderr)] {
            assert!(
                !body.contains(secret),
                "`mamba {}` echoed the `auth` probe's own credential material on {stream}: the \
                 literal `{secret}` appears in what the validator printed\n{}",
                run.args.join(" "),
                run.render()
            );
        }
    }
}

#[test]
fn pkgmgr_validate_reports_the_auth_family_passing_offline() {
    require_python3();
    let sandbox = Sandbox::new();

    // ---------------------------------------------------------------- json
    let json_run = sandbox.run("json", &["pkgmgr-validate", "--json"]);

    // Behaviour, and the reason this work item exists: the authenticated add
    // the `auth` probe performs must be accepted against the lock it wrote.
    let auth = family_object(&json_run.stdout, FAMILY, &json_run);
    assert_eq!(
        field(&auth, "outcome", FAMILY, &json_run),
        "\"pass\"",
        "the `{FAMILY}` family must pass: its authenticated `add` against the probe's own mock \
         registry must be accepted against the lock that add wrote\n{}",
        json_run.render()
    );
    assert_eq!(
        field(&auth, "failed", FAMILY, &json_run),
        "0",
        "the `{FAMILY}` family must report no failure\n{}",
        json_run.render()
    );

    // No family filter exists, so the exit status is the verdict over the
    // whole required set.
    assert_eq!(
        json_run.code,
        Some(0),
        "`mamba pkgmgr-validate --json` must exit 0 once every required family passes\n{}",
        json_run.render()
    );

    // A green bought by removing a family is not a green.
    let declared = family_names(&json_run.stdout);
    assert_eq!(
        declared.len(),
        REQUIRED_FAMILY_COUNT,
        "`pkgmgr-validate` must still report all {REQUIRED_FAMILY_COUNT} required families; it \
         reported {:?}\n{}",
        declared,
        json_run.render()
    );
    assert!(
        declared.iter().any(|name| name == FAMILY),
        "the required family set must still contain `{FAMILY}`; it was {:?}\n{}",
        declared,
        json_run.render()
    );

    // Security: the run that produced this verdict never enabled the network.
    assert_eq!(
        unique_field(&json_run.stdout, "include_live_network", &json_run),
        "false",
        "this case never passes `--include-live-network`, and the validator must report the \
         offline default back\n{}",
        json_run.render()
    );
    assert_no_credential_leak(&json_run);

    // --------------------------------------------------------------- human
    let human_run = sandbox.run("human", &["pkgmgr-validate"]);
    assert!(
        human_run.stderr.contains(EXPECTED_HUMAN_LINE),
        "the human report must carry `{EXPECTED_HUMAN_LINE}`\n{}",
        human_run.render()
    );
    assert!(
        human_run.stderr.contains(EXPECTED_SUMMARY),
        "the human report must end at `{EXPECTED_SUMMARY}`\n{}",
        human_run.render()
    );
    assert_eq!(
        human_run.code,
        Some(0),
        "`mamba pkgmgr-validate` must exit 0 once every required family passes\n{}",
        human_run.render()
    );
    assert_no_credential_leak(&human_run);
}
