//! Black-box contract: `mamba sync` must remove the distributions the lock no
//! longer pins, and `mamba sync --check` must report them instead of calling
//! the environment synchronized.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary over a throwaway directory tree
//! and reads only what the binary wrote or what the project's own interpreter
//! reports. Both distributions are real wheels built by the product's own
//! `WheelBuilder`, frozen by `mamba index build`, locked by `mamba add`, and
//! installed by `mamba sync`, so the state this case prunes from is one the
//! product produced end to end. Nothing is hand-placed in the environment,
//! and nothing is hand-deleted from it: the only thing that ever removes an
//! installed distribution here is `mamba sync`.
//!
//! # The observation point
//!
//! `purelib` is `<venv>/lib/python<X.Y>/site-packages`, derived from the
//! `version` key the environment's own `pyvenv.cfg` carries, and it is
//! cross-checked against `sysconfig.get_paths()["purelib"]` as reported by
//! the environment's interpreter before any assertion runs. A tree where
//! those two disagree fails as a fixture error naming both, so an assertion
//! about a missing `b-1.0.dist-info` can never pass by pointing at a
//! directory the product never used.
//!
//! # Why today's tree cannot pass
//!
//! Today `plan_install` returns only the pending set — locked but not
//! installed — and nothing enumerates installed distributions the lock has
//! stopped pinning. After `mamba remove b` the pending set is empty, so
//! `sync --check` prints `environment is synchronized with mamba.lock` and
//! exits 0 while `b` is still installed and still importable, and `sync`
//! prints its `no_op` line over the same divergence. Four assertions
//! separately refuse that shape:
//!
//! 1. `sync --check` exits non-zero and names `b==1.0`.
//! 2. `mamba run -- python -c "import b"` fails after `sync`.
//! 3. `b-1.0.dist-info` is gone from `purelib`.
//! 4. `a-1.0.dist-info`, `a/`, and the interpreter it imports through are all
//!    still there — the removal is scoped, not a wipe.
//!
//! # Facets
//!
//! - **Behavior**: one `mamba remove` makes an installed distribution
//!   extraneous; `--check` reports it, `sync` removes it, the module stops
//!   importing, the distribution the lock still pins keeps working, and the
//!   next `sync` is the documented `no_op` while `--check` goes back to
//!   reporting a synchronized environment. Convergence is reached once and
//!   then stays reached.
//! - **Security (the lock is the authority for what may execute)**: an
//!   environment that keeps running code the lock no longer authorizes is the
//!   fail-open state this work item removes — `--check` reporting
//!   `synchronized` over it is the silent half, and `import b` still
//!   succeeding after `sync` is the loud half. Both are asserted. The
//!   containment side is asserted too: `--check` is a read-only verb, so the
//!   distribution must still be present after it refuses; and the removal is
//!   bounded to the distribution's own files, so `a`, `pyvenv.cfg`, the
//!   interpreter, and `bin/` must all survive it byte-for-byte. A prune that
//!   converges by recreating `.venv` fails on the `pyvenv.cfg` digest.
//! - **Performance**: the work item names no budget, so this case asserts no
//!   timing.
//!
//! # Hermeticity
//!
//! No network, ever: both wheels are built in-process and the index is a
//! local directory. `HOME` and `MAMBA_CACHE_DIR` are pinned into the temp
//! tree; `MAMBA_FROZEN_INDEX`, `MAMBA_INDEX_URL`, `MAMBA_JOBS`,
//! `XDG_CACHE_HOME`, `VIRTUAL_ENV`, and `PYTHONPATH` are removed so no
//! ambient value can supply an answer. Nothing is skipped and nothing is
//! `#[ignore]`d: a host with no `python3` fails the case naming it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mamba::pkgmanage::pkgmgr::wheel_build::{
    compose_filename, CoreMetadata, WheelBuilder, WheelMetadata,
};
use sha2::{Digest, Sha256};

/// The distribution the lock keeps pinning. It must survive the prune.
const KEPT: &str = "a";
/// The distribution `mamba remove` unpins. It must not survive the prune.
const PRUNED: &str = "b";
const VERSION: &str = "1.0";
const KEPT_DIST_INFO: &str = "a-1.0.dist-info";
const PRUNED_DIST_INFO: &str = "b-1.0.dist-info";
/// How `sync --check` must name the distribution the lock no longer pins.
const PRUNED_PIN: &str = "b==1.0";
/// The pin that must not appear in that report: `a` is installed and still
/// locked, so it is neither pending nor extraneous.
const KEPT_PIN: &str = "a==1.0";
/// The documented idempotence signal, unchanged by this work item.
const NO_OP: &str = "no_op: environment already in sync with mamba.lock";
/// The documented `--check` success line, unchanged by this work item.
const SYNCHRONIZED: &str = "environment is synchronized with mamba.lock";

/// Each module's `who()` returns its own name, so a successful import proves
/// the wheel's own bytes are what the interpreter loaded.
fn module_body(name: &str) -> String {
    format!("def who():\n    return {name:?}\n")
}

/// The binary under test. `CARGO_BIN_EXE_*` is resolved by cargo at compile
/// time, so a missing binary is a build failure rather than a skipped case.
fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

/// Run the binary in `cwd` with an environment that cannot reach a registry,
/// a user-global cache, or an ambient import path.
fn run(cwd: &Path, home: &Path, args: &[&str]) -> Output {
    Command::new(mamba_bin())
        .args(args)
        .current_dir(cwd)
        .env("HOME", home)
        .env("MAMBA_CACHE_DIR", home.join("cache"))
        .env_remove("MAMBA_FROZEN_INDEX")
        .env_remove("MAMBA_INDEX_URL")
        .env_remove("MAMBA_JOBS")
        .env_remove("XDG_CACHE_HOME")
        .env_remove("VIRTUAL_ENV")
        .env_remove("PYTHONPATH")
        .output()
        .unwrap_or_else(|e| panic!("spawn {} {args:?}: {e}", mamba_bin().display()))
}

fn render(args: &[&str], out: &Output) -> String {
    format!(
        "`mamba {}` exited {:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        args.join(" "),
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    )
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

/// Everything the run said, on either stream. Which stream carries a report
/// is not what this case judges; that the report exists is.
fn combined(out: &Output) -> String {
    format!("{}{}", stdout_of(out), stderr_of(out))
}

/// Require exit 0, reporting the whole invocation when it is not.
fn expect_ok(step: &str, args: &[&str], out: &Output) {
    assert!(
        out.status.success(),
        "step `{step}` must succeed: {}",
        render(args, out)
    );
}

/// Require a non-zero exit, and hand back everything the run said so the
/// caller can assert on the refusal text.
fn expect_failure(step: &str, args: &[&str], out: &Output) -> String {
    assert!(
        !out.status.success(),
        "step `{step}` must fail, not succeed: {}",
        render(args, out)
    );
    combined(out)
}

/// Build one real, importable wheel through the product's own wheel builder.
fn build_wheel(out_dir: &Path, name: &str, version: &str) -> PathBuf {
    let filename = compose_filename(name, version, "py3", "none", "any");
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-sync-prune");
    wheel_meta.tags.push("py3-none-any".into());
    let core_meta = CoreMetadata::new(name, version);
    let mut builder = WheelBuilder::new(filename, wheel_meta, core_meta);
    builder.add_file(format!("{name}/__init__.py"), module_body(name));
    let wheel = builder
        .build_to_dir(out_dir)
        .unwrap_or_else(|e| panic!("fixture: build wheel {name}-{version}: {e:?}"));
    let expected = format!("{name}-{version}-py3-none-any.whl");
    assert_eq!(
        wheel.file_name().and_then(|n| n.to_str()),
        Some(expected.as_str()),
        "fixture: the wheel builder named the artifact {}",
        wheel.display()
    );
    wheel
}

fn sha256_file(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    format!("{:x}", hasher.finalize())
}

fn canon(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn path_arg(path: &Path) -> String {
    path.to_str()
        .unwrap_or_else(|| panic!("fixture: path {} is not utf-8", path.display()))
        .to_string()
}

fn listing(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

/// The `python3` this host offers, resolved to the interpreter it actually
/// runs. `python3` on `PATH` is commonly a version-manager shim whose own
/// resolution depends on `HOME`, and this case pins `HOME` into a temp tree;
/// asking the shim for `sys.executable` gets the real interpreter, which
/// `mamba venv --python` can seed from under any environment. A host with no
/// `python3` fails here, naming it — the case never skips.
fn resolve_python3() -> PathBuf {
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
        panic!("this case needs `python3` on PATH and found none in {path_var:?}")
    });
    let out = Command::new(&candidate)
        .args(["-c", "import sys; print(sys.executable)"])
        .output()
        .unwrap_or_else(|e| panic!("spawn {}: {e}", candidate.display()));
    assert!(
        out.status.success(),
        "`{} -c 'print(sys.executable)'` failed: {}",
        candidate.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    let real = PathBuf::from(stdout_of(&out).trim());
    assert!(
        real.is_file(),
        "`{}` reported the interpreter {}, which is not a file",
        candidate.display(),
        real.display()
    );
    real
}

/// One scaffolded project: `mamba init`, a real PEP 405 environment from the
/// host `python3`, and the `purelib` that environment installs into.
struct Case {
    _root: tempfile::TempDir,
    root: PathBuf,
    project: PathBuf,
    home: PathBuf,
    purelib: PathBuf,
}

impl Case {
    fn start() -> Case {
        let root_dir = tempfile::tempdir().expect("create temp root");
        let root = root_dir.path().to_path_buf();
        let project = root.join("project");
        let home = root.join("home");
        for dir in [&project, &home] {
            std::fs::create_dir_all(dir)
                .unwrap_or_else(|e| panic!("create {}: {e}", dir.display()));
        }

        let init_args = ["init"];
        let out = run(&project, &home, &init_args);
        expect_ok("init", &init_args, &out);
        assert!(
            project.join("mamba.toml").is_file(),
            "fixture: `mamba init` wrote no mamba.toml in {}",
            project.display()
        );

        let python3 = resolve_python3();
        let python_arg = path_arg(&python3);
        let venv_args = ["venv", "--python", python_arg.as_str()];
        let out = run(&project, &home, &venv_args);
        expect_ok("venv --python <python3>", &venv_args, &out);

        let case = Case {
            _root: root_dir,
            root,
            project,
            home,
            purelib: PathBuf::new(),
        };
        let purelib = case.derived_purelib();
        let reported = case.reported_purelib();
        assert_eq!(
            canon(&purelib),
            canon(&reported),
            "fixture: `<venv>/lib/python<X.Y>/site-packages` derived from pyvenv.cfg is {}, but \
             the environment's own interpreter installs into {}; the case would be asserting \
             about a directory the product never uses",
            purelib.display(),
            reported.display()
        );
        assert!(
            purelib.is_dir(),
            "fixture: {} is not a directory",
            purelib.display()
        );
        Case { purelib, ..case }
    }

    fn venv(&self) -> PathBuf {
        self.project.join(".venv")
    }

    fn pyvenv_cfg(&self) -> PathBuf {
        self.venv().join("pyvenv.cfg")
    }

    /// `<venv>/lib/python<X.Y>/site-packages`, with `<X.Y>` read from the
    /// `version` the environment's own `pyvenv.cfg` records.
    fn derived_purelib(&self) -> PathBuf {
        let cfg_path = self.pyvenv_cfg();
        let body = std::fs::read_to_string(&cfg_path)
            .unwrap_or_else(|e| panic!("fixture: read {}: {e}", cfg_path.display()));
        let version = body
            .lines()
            .filter_map(|line| line.split_once('='))
            .find(|(key, _)| matches!(key.trim(), "version" | "version_info"))
            .map(|(_, value)| value.trim().to_string())
            .unwrap_or_else(|| {
                panic!(
                    "fixture: {} records no `version`\n--- pyvenv.cfg ---\n{body}",
                    cfg_path.display()
                )
            });
        let mut parts = version.split('.');
        let major = parts.next().unwrap_or_default().trim().to_string();
        let minor = parts.next().unwrap_or_default().trim().to_string();
        assert!(
            !major.is_empty() && !minor.is_empty(),
            "fixture: {} records the unparseable version {version:?}",
            cfg_path.display()
        );
        self.venv()
            .join("lib")
            .join(format!("python{major}.{minor}"))
            .join("site-packages")
    }

    /// `.venv/bin/python` (POSIX) or `.venv/Scripts/python.exe` (Windows), as
    /// `python -m venv` laid it down.
    fn venv_python(&self) -> PathBuf {
        for candidate in [
            self.venv().join("bin").join("python"),
            self.venv().join("Scripts").join("python.exe"),
        ] {
            if candidate.is_file() {
                return candidate;
            }
        }
        panic!(
            "fixture: `mamba venv` left no interpreter under {}",
            self.venv().display()
        );
    }

    /// Where the environment's own interpreter says a pure-Python
    /// distribution belongs. Used to cross-check the derivation above.
    fn reported_purelib(&self) -> PathBuf {
        let python = self.venv_python();
        let out = Command::new(&python)
            .args([
                "-c",
                "import sysconfig; print(sysconfig.get_paths()['purelib'])",
            ])
            .current_dir(&self.project)
            .env("HOME", &self.home)
            .env_remove("PYTHONPATH")
            .env_remove("VIRTUAL_ENV")
            .output()
            .unwrap_or_else(|e| panic!("spawn {}: {e}", python.display()));
        assert!(
            out.status.success(),
            "fixture: `{} -c 'print purelib'` failed: {}",
            python.display(),
            String::from_utf8_lossy(&out.stderr)
        );
        PathBuf::from(stdout_of(&out).trim())
    }

    fn run(&self, args: &[&str]) -> Output {
        run(&self.project, &self.home, args)
    }

    fn dist_info(&self, dir_name: &str) -> PathBuf {
        self.purelib.join(dir_name)
    }

    fn module_dir(&self, name: &str) -> PathBuf {
        self.purelib.join(name)
    }

    /// `mamba run -- python -c "import <name>; print(<name>.who())"`.
    fn import(&self, name: &str) -> Output {
        let probe = format!("import {name}; print({name}.who())");
        self.run(&["run", "--", "python", "-c", probe.as_str()])
    }

    /// Bring `a` and `b` to installed-and-importable through the product's
    /// own verbs. Everything here already holds on the tree this case was
    /// written against; a failure below is a broken fixture, not the
    /// behavior under test.
    fn install_both(&self) {
        let wheels = self.root.join("wheels");
        let index = self.root.join("index");
        let kept = build_wheel(&wheels, KEPT, VERSION);
        let pruned = build_wheel(&wheels, PRUNED, VERSION);

        let index_arg = path_arg(&index);
        let kept_arg = path_arg(&kept);
        let pruned_arg = path_arg(&pruned);
        let build_args = [
            "index",
            "build",
            "--out",
            index_arg.as_str(),
            kept_arg.as_str(),
            pruned_arg.as_str(),
        ];
        let out = self.run(&build_args);
        expect_ok("fixture: index build", &build_args, &out);

        for name in [KEPT, PRUNED] {
            let add_args = ["add", name, "--index", index_arg.as_str()];
            let out = self.run(&add_args);
            expect_ok(&format!("fixture: add {name} --index"), &add_args, &out);
        }

        let sync_args = ["sync"];
        let out = self.run(&sync_args);
        expect_ok("fixture: first sync", &sync_args, &out);

        for name in [KEPT, PRUNED] {
            let dist_info = self.dist_info(&format!("{name}-{VERSION}.dist-info"));
            assert!(
                dist_info.is_dir(),
                "fixture: `mamba sync` must install {name}=={VERSION} into {}; {} is missing. \
                 Present: {:?}",
                self.purelib.display(),
                dist_info.display(),
                listing(&self.purelib)
            );
            let probe_args = ["run", "--", "python", "-c", "import <name>"];
            let out = self.import(name);
            expect_ok(
                &format!("fixture: run -- python -c 'import {name}'"),
                &probe_args,
                &out,
            );
            assert_eq!(
                stdout_of(&out).trim(),
                name,
                "fixture: the imported module must be the wheel's own source for {name}: {}",
                render(&probe_args, &out)
            );
        }
    }
}

#[test]
fn sync_prunes_the_distribution_the_lock_stopped_pinning_and_check_reports_it() {
    let case = Case::start();
    case.install_both();

    // The environment identity the prune must not disturb: recreating `.venv`
    // to converge would change these bytes.
    let cfg_before = std::fs::read(case.pyvenv_cfg())
        .unwrap_or_else(|e| panic!("read {}: {e}", case.pyvenv_cfg().display()));
    let cfg_digest_before = sha256_file(&case.pyvenv_cfg());
    let interpreter_before = case.venv_python();

    // Step 1 — the lock stops pinning `b`. Nothing here touches `.venv`.
    let remove_args = ["remove", PRUNED];
    let out = case.run(&remove_args);
    expect_ok("step 1 (remove b)", &remove_args, &out);
    let lock = std::fs::read_to_string(case.project.join("mamba.lock"))
        .unwrap_or_else(|e| panic!("read mamba.lock: {e}"));
    assert!(
        !lock.contains(&format!("name = \"{PRUNED}\"")),
        "step 1 (remove b): the lock must stop pinning {PRUNED}\n--- mamba.lock ---\n{lock}"
    );

    // Step 2 — `sync --check` must refuse the divergence and name it. Today
    // the pending set is empty, so this prints `environment is synchronized
    // with mamba.lock` and exits 0 over a still-installed `b`.
    let check_args = ["sync", "--check"];
    let out = case.run(&check_args);
    let said = expect_failure("step 2 (sync --check after remove)", &check_args, &out);
    assert!(
        said.contains(PRUNED_PIN),
        "step 2 (sync --check after remove): the report must name {PRUNED_PIN} as extraneous — \
         it is installed and the lock no longer pins it: {}",
        render(&check_args, &out)
    );
    assert!(
        !said.contains(KEPT_PIN),
        "step 2 (sync --check after remove): {KEPT_PIN} is installed and still locked, so it is \
         neither pending nor extraneous and must not be reported: {}",
        render(&check_args, &out)
    );

    // Step 3 — `--check` reports without mutating: the distribution it just
    // named is still there.
    assert!(
        case.dist_info(PRUNED_DIST_INFO).is_dir(),
        "step 3 (--check does not mutate): `sync --check` must report the extraneous \
         distribution, not remove it; {} is gone. Present: {:?}",
        case.dist_info(PRUNED_DIST_INFO).display(),
        listing(&case.purelib)
    );

    // Step 4 — `sync` converges by removing it.
    let sync_args = ["sync"];
    let out = case.run(&sync_args);
    expect_ok("step 4 (sync prunes)", &sync_args, &out);

    // Step 5 — the module the lock no longer pins stops importing.
    let import_pruned_args = ["run", "--", "python", "-c", "import b"];
    let out = case.import(PRUNED);
    let said = expect_failure(
        "step 5 (import b after the prune)",
        &import_pruned_args,
        &out,
    );
    assert!(
        said.contains("No module named"),
        "step 5 (import b after the prune): the import must fail because {PRUNED} is gone: {}",
        render(&import_pruned_args, &out)
    );

    // Step 6 — the distribution the lock still pins is untouched and still
    // imports its own source.
    let import_kept_args = ["run", "--", "python", "-c", "import a"];
    let out = case.import(KEPT);
    expect_ok("step 6 (import a after the prune)", &import_kept_args, &out);
    assert_eq!(
        stdout_of(&out).trim(),
        KEPT,
        "step 6 (import a after the prune): the prune must leave {KEPT}'s own module in place: {}",
        render(&import_kept_args, &out)
    );

    // Step 7 — the environment's own directory says the same thing: what
    // RECORD listed for `b` is gone, what belongs to `a` is not.
    assert!(
        !case.dist_info(PRUNED_DIST_INFO).exists(),
        "step 7 (purelib after the prune): {} must be gone. Present under {}: {:?}",
        case.dist_info(PRUNED_DIST_INFO).display(),
        case.purelib.display(),
        listing(&case.purelib)
    );
    assert!(
        !case.module_dir(PRUNED).exists(),
        "step 7 (purelib after the prune): the pruned distribution's own module {} must be gone \
         too — removing the dist-info alone leaves the code importable. Present under {}: {:?}",
        case.module_dir(PRUNED).display(),
        case.purelib.display(),
        listing(&case.purelib)
    );
    assert!(
        case.dist_info(KEPT_DIST_INFO).is_dir(),
        "step 7 (purelib after the prune): {} must survive a prune scoped to {PRUNED}. Present \
         under {}: {:?}",
        case.dist_info(KEPT_DIST_INFO).display(),
        case.purelib.display(),
        listing(&case.purelib)
    );
    assert!(
        case.module_dir(KEPT).is_dir(),
        "step 7 (purelib after the prune): {} must survive a prune scoped to {PRUNED}. Present \
         under {}: {:?}",
        case.module_dir(KEPT).display(),
        case.purelib.display(),
        listing(&case.purelib)
    );

    // Step 8 — the environment itself was never recreated: same `pyvenv.cfg`
    // bytes, same interpreter, same `bin/`.
    let cfg_after = std::fs::read(case.pyvenv_cfg())
        .unwrap_or_else(|e| panic!("read {}: {e}", case.pyvenv_cfg().display()));
    let cfg_digest_after = sha256_file(&case.pyvenv_cfg());
    assert_eq!(
        cfg_after,
        cfg_before,
        "step 8 (the venv is not recreated): {} changed across the prune \
         (sha256 {cfg_digest_before} -> {cfg_digest_after})\n--- before ---\n{}\n--- after ---\n{}",
        case.pyvenv_cfg().display(),
        String::from_utf8_lossy(&cfg_before),
        String::from_utf8_lossy(&cfg_after),
    );
    assert!(
        interpreter_before.is_file(),
        "step 8 (the venv is not recreated): the environment's interpreter {} must survive the \
         prune",
        interpreter_before.display()
    );
    assert!(
        interpreter_before
            .parent()
            .map(|bin| bin.is_dir())
            .unwrap_or(false),
        "step 8 (the venv is not recreated): the environment's script directory beside {} must \
         survive the prune",
        interpreter_before.display()
    );

    // Step 9 — converged: the next `sync` is the documented no-op, and
    // `--check` reports a synchronized environment again.
    let out = case.run(&sync_args);
    expect_ok("step 9 (second sync)", &sync_args, &out);
    assert!(
        stderr_of(&out).contains(NO_OP),
        "step 9 (second sync): once nothing is pending and nothing is extraneous, `mamba sync` \
         must be the documented no-op: {}",
        render(&sync_args, &out)
    );
    let out = case.run(&check_args);
    expect_ok("step 9 (sync --check after the prune)", &check_args, &out);
    assert!(
        stdout_of(&out).contains(SYNCHRONIZED),
        "step 9 (sync --check after the prune): the environment must read back as synchronized \
         once the extraneous distribution is gone: {}",
        render(&check_args, &out)
    );
}
