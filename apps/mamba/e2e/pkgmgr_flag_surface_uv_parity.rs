//! Black-box contract: the day-to-day `uv` flags a project workflow leans on
//! exist on `mamba` and mean what they mean on `uv`.
//!
//! # What this case owns
//!
//! One case per flag family, each driving the built `mamba` binary over a
//! throwaway tree and reading only what the binary wrote or what the
//! project's own interpreter reports:
//!
//! - `add --dev` / `--group` / `--optional` land the requirement in the PEP
//!   735 group or the PEP 621 extra, and `mamba.lock` records which closure
//!   each package belongs to (`project`, `groups`, `extras`).
//! - `sync` installs the project closure plus the `dev` group by default;
//!   `--no-dev`, `--extra`, `--all-extras`, and `--all-groups` change the
//!   selection and the environment converges on exactly that selection.
//! - `sync --locked` refuses a lock whose `input_hash` no longer matches the
//!   manifest; `sync --frozen` syncs from the lock as it stands.
//! - `lock` keeps the versions the existing lock already records whenever
//!   they still satisfy the manifest; `--upgrade-package NAME` frees one
//!   name and `--upgrade` frees them all.
//! - `add -r FILE` reads a pip requirements file; `remove` takes several
//!   names and the same scoping flags as `add`.
//! - `run --no-project` skips the sync preflight and the `.venv` injection;
//!   `run --python PATH` runs the file on that interpreter.
//!
//! # Hermeticity
//!
//! No network: every wheel is built in-process by the product's own
//! `WheelBuilder` and every index is a local directory. `HOME` and
//! `MAMBA_CACHE_DIR` are pinned into the temp tree; `MAMBA_FROZEN_INDEX`,
//! `MAMBA_INDEX_URL`, `MAMBA_JOBS`, `XDG_CACHE_HOME`, `VIRTUAL_ENV`, and
//! `PYTHONPATH` are removed. Nothing is skipped: a host with no `python3`
//! fails the case naming it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mamba::pkgmanage::pkgmgr::wheel_build::{
    compose_filename, CoreMetadata, WheelBuilder, WheelMetadata,
};

const LOCKED_REFUSAL: &str = "`--locked` was provided";

fn module_body(name: &str) -> String {
    format!("def who():\n    return {name:?}\n")
}

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

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

fn combined(out: &Output) -> String {
    format!("{}{}", stdout_of(out), String::from_utf8_lossy(&out.stderr))
}

fn expect_ok(step: &str, args: &[&str], out: &Output) {
    assert!(
        out.status.success(),
        "step `{step}` must succeed: {}",
        render(args, out)
    );
}

fn expect_failure(step: &str, args: &[&str], out: &Output) -> String {
    assert!(
        !out.status.success(),
        "step `{step}` must fail, not succeed: {}",
        render(args, out)
    );
    combined(out)
}

fn build_wheel(out_dir: &Path, name: &str, version: &str) -> PathBuf {
    let filename = compose_filename(name, version, "py3", "none", "any");
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-flag-surface");
    wheel_meta.tags.push("py3-none-any".into());
    let core_meta = CoreMetadata::new(name, version);
    let mut builder = WheelBuilder::new(filename, wheel_meta, core_meta);
    builder.add_file(format!("{name}/__init__.py"), module_body(name));
    builder
        .build_to_dir(out_dir)
        .unwrap_or_else(|e| panic!("fixture: build wheel {name}-{version}: {e:?}"))
}

fn path_arg(path: &Path) -> String {
    path.to_str()
        .unwrap_or_else(|| panic!("fixture: path {} is not utf-8", path.display()))
        .to_string()
}

fn canon(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn write(path: &Path, body: &str) {
    std::fs::write(path, body).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}

/// Replace exactly one occurrence, refusing a manifest that does not carry
/// the expected text so an edit never silently misses.
fn replace_once(path: &Path, from: &str, to: &str) {
    let body = read(path);
    assert_eq!(
        body.matches(from).count(),
        1,
        "fixture: {} must contain {from:?} exactly once\n--- file ---\n{body}",
        path.display()
    );
    write(path, &body.replacen(from, to, 1));
}

/// The `[[package]]` block for `name` in a rendered lock, so membership
/// assertions read one package and not another's fields.
fn lock_block<'a>(lock: &'a str, name: &str) -> &'a str {
    let header = format!("[[package]]\nname = \"{name}\"\n");
    let start = lock
        .find(&header)
        .unwrap_or_else(|| panic!("lock has no block for {name}\n--- mamba.lock ---\n{lock}"));
    let rest = &lock[start + header.len()..];
    let end = rest.find("[[package]]").unwrap_or(rest.len());
    &rest[..end]
}

fn resolve_python3() -> PathBuf {
    let path_var = std::env::var_os("PATH")
        .unwrap_or_else(|| panic!("this case needs `python3`, and PATH is unset"));
    let candidate = std::env::split_paths(&path_var)
        .flat_map(|dir| [dir.join("python3"), dir.join("python3.exe")])
        .find(|probe| probe.is_file())
        .unwrap_or_else(|| {
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
        "python3 reported {}, not a file",
        real.display()
    );
    real
}

/// One scaffolded project with a frozen index holding `a`, `b`, and `c` at
/// 1.0, and — once `with_venv` ran — a real PEP 405 environment.
struct Case {
    _root: tempfile::TempDir,
    root: PathBuf,
    project: PathBuf,
    home: PathBuf,
    index: PathBuf,
    python3: PathBuf,
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
        let out = run(&project, &home, &["init"]);
        expect_ok("init", &["init"], &out);

        let index = root.join("index");
        let wheels = root.join("wheels");
        let built: Vec<String> = ["a", "b", "c"]
            .iter()
            .map(|name| path_arg(&build_wheel(&wheels, name, "1.0")))
            .collect();
        let case = Case {
            _root: root_dir,
            root,
            project,
            home,
            index,
            python3: resolve_python3(),
        };
        case.build_index(&case.index.clone(), &built);
        case
    }

    fn build_index(&self, out: &Path, wheels: &[String]) {
        let out_arg = path_arg(out);
        let mut args = vec!["index", "build", "--out", out_arg.as_str()];
        args.extend(wheels.iter().map(String::as_str));
        let result = self.run(&args);
        expect_ok("fixture: index build", &args, &result);
    }

    fn with_venv(&self) {
        let python_arg = path_arg(&self.python3);
        let args = ["venv", "--python", python_arg.as_str()];
        let out = self.run(&args);
        expect_ok("fixture: venv --python <python3>", &args, &out);
    }

    fn run(&self, args: &[&str]) -> Output {
        run(&self.project, &self.home, args)
    }

    fn ok(&self, step: &str, args: &[&str]) -> Output {
        let out = self.run(args);
        expect_ok(step, args, &out);
        out
    }

    fn index_arg(&self) -> String {
        path_arg(&self.index)
    }

    fn manifest(&self) -> PathBuf {
        self.project.join("pyproject.toml")
    }

    fn lock(&self) -> String {
        read(&self.project.join("mamba.lock"))
    }

    fn add(&self, step: &str, extra_args: &[&str]) {
        let index = self.index_arg();
        let mut args = vec!["add"];
        args.extend_from_slice(extra_args);
        args.extend(["--index", index.as_str()]);
        self.ok(step, &args);
    }

    /// The `dist-info` names the environment's own interpreter can see.
    fn installed(&self) -> Vec<String> {
        let out = self.ok(
            "probe: list installed",
            &[
                "run",
                "--",
                "python",
                "-c",
                "import importlib.metadata as m; print(' '.join(sorted(d.metadata['Name'] \
                 for d in m.distributions() if d.metadata['Name'] in ('a', 'b', 'c'))))",
            ],
        );
        stdout_of(&out)
            .split_whitespace()
            .map(str::to_string)
            .collect()
    }

    /// Seed the project with `a` (project), `b` (`dev` group), and `c`
    /// (extra `fast`), the three closures every selection case reads.
    fn seed_groups(&self) {
        self.add("add a", &["a"]);
        self.add("add --dev b", &["--dev", "b"]);
        self.add("add --optional fast c", &["--optional", "fast", "c"]);
    }
}

#[test]
fn add_scopes_land_in_the_manifest_and_the_lock_records_each_closure() {
    let case = Case::start();
    case.seed_groups();

    let manifest = read(&case.manifest());
    for (section, member) in [
        ("[project]", "\"a==1.0\""),
        ("[dependency-groups]", "dev = [\n    \"b==1.0\",\n]"),
        (
            "[project.optional-dependencies]",
            "fast = [\n    \"c==1.0\",\n]",
        ),
    ] {
        assert!(
            manifest.contains(section) && manifest.contains(member),
            "pyproject.toml must carry {member:?} under {section}\n--- pyproject.toml ---\n{manifest}"
        );
    }
    assert!(
        !manifest.contains("dev-dependencies"),
        "the retired mamba.toml spelling must not reappear\n{manifest}"
    );

    let lock = case.lock();
    let a = lock_block(&lock, "a");
    assert!(
        !a.contains("project = false") && !a.contains("groups =") && !a.contains("extras ="),
        "a is a project dependency: no membership fields\n--- block ---\n{a}"
    );
    let b = lock_block(&lock, "b");
    assert!(
        b.contains("project = false\n") && b.contains("groups = [\"dev\"]\n"),
        "b belongs to the dev group only\n--- block ---\n{b}"
    );
    let c = lock_block(&lock, "c");
    assert!(
        c.contains("project = false\n") && c.contains("extras = [\"fast\"]\n"),
        "c belongs to the fast extra only\n--- block ---\n{c}"
    );

    // `--group` with a fresh name creates the group; membership follows.
    case.add("add --group docs c", &["--group", "docs", "c"]);
    let manifest = read(&case.manifest());
    assert!(
        manifest.contains("docs = [\n    \"c==1.0\",\n]"),
        "--group docs must create the group\n{manifest}"
    );
    let lock = case.lock();
    let c = lock_block(&lock, "c");
    assert!(
        c.contains("groups = [\"docs\"]\n") && c.contains("extras = [\"fast\"]\n"),
        "c now belongs to the docs group and the fast extra\n--- block ---\n{c}"
    );
}

#[test]
fn sync_selection_follows_uv_defaults_and_converges_on_it() {
    let case = Case::start();
    case.with_venv();
    case.seed_groups();

    case.ok("sync (default)", &["sync"]);
    assert_eq!(
        case.installed(),
        ["a", "b"],
        "default = project + dev group"
    );

    case.ok("sync --no-dev", &["sync", "--no-dev"]);
    assert_eq!(case.installed(), ["a"], "--no-dev prunes the dev group");

    case.ok("sync --extra fast", &["sync", "--extra", "fast"]);
    assert_eq!(
        case.installed(),
        ["a", "b", "c"],
        "--extra adds the extra to the default"
    );

    case.ok(
        "sync --no-dev --all-extras",
        &["sync", "--no-dev", "--all-extras"],
    );
    assert_eq!(case.installed(), ["a", "c"], "--all-extras without dev");

    case.ok("sync --all-groups", &["sync", "--all-groups"]);
    assert_eq!(
        case.installed(),
        ["a", "b"],
        "--all-groups: dev is the only group"
    );

    case.ok("sync --dev", &["sync", "--dev"]);
    assert_eq!(case.installed(), ["a", "b"], "--dev restates the default");

    let args = ["sync", "--no-dev", "--dev"];
    let out = case.run(&args);
    expect_failure("sync --no-dev --dev conflict", &args, &out);
}

#[test]
fn sync_locked_refuses_a_stale_lock_while_frozen_syncs_from_it() {
    let case = Case::start();
    case.with_venv();
    case.add("add a", &["a"]);
    case.ok("sync --locked (fresh lock)", &["sync", "--locked"]);

    // A comment does not change the lock inputs.
    let manifest = case.manifest();
    write(&manifest, &format!("{}\n# a note\n", read(&manifest)));
    case.ok("sync --locked (comment only)", &["sync", "--locked"]);

    // A new requirement does.
    replace_once(
        &manifest,
        "dependencies = [\n    \"a==1.0\",\n]",
        "dependencies = [\n    \"a==1.0\",\n    \"b==1.0\",\n]",
    );
    let args = ["sync", "--locked"];
    let out = case.run(&args);
    let said = expect_failure("sync --locked (stale)", &args, &out);
    assert!(
        said.contains(LOCKED_REFUSAL) && said.contains("mamba lock"),
        "the refusal must name --locked and the command that fixes it: {}",
        render(&args, &out)
    );
    assert_eq!(
        case.installed(),
        ["a"],
        "--locked must not touch the environment"
    );

    case.ok("sync --frozen (stale)", &["sync", "--frozen"]);
    assert_eq!(
        case.installed(),
        ["a"],
        "--frozen syncs what the lock records"
    );

    let args = ["sync", "--locked", "--frozen"];
    let out = case.run(&args);
    expect_failure("sync --locked --frozen conflict", &args, &out);

    // `mamba lock` repairs it; `--locked` is satisfied again.
    let index = case.index_arg();
    case.ok("lock", &["lock", "--index", index.as_str()]);
    case.ok("sync --locked (repaired)", &["sync", "--locked"]);
    assert_eq!(case.installed(), ["a", "b"]);
}

#[test]
fn lock_keeps_recorded_pins_until_upgrade_frees_them() {
    let case = Case::start();
    // The manifest admits any 1.x or later `a`; only the index decides.
    replace_once(
        &case.manifest(),
        "dependencies = []",
        "dependencies = [\n    \"a>=1\",\n]",
    );
    let old = case.index_arg();
    case.ok("lock (1.0 only)", &["lock", "--index", old.as_str()]);
    assert!(lock_block(&case.lock(), "a").contains("version = \"1.0\""));

    let newer = case.root.join("index-newer");
    let wheels = case.root.join("wheels-newer");
    let built = vec![
        path_arg(&build_wheel(&wheels, "a", "1.0")),
        path_arg(&build_wheel(&wheels, "a", "2.0")),
    ];
    case.build_index(&newer, &built);
    let new = path_arg(&newer);

    case.ok("lock (2.0 available)", &["lock", "--index", new.as_str()]);
    assert!(
        lock_block(&case.lock(), "a").contains("version = \"1.0\""),
        "a recorded pin that still satisfies the manifest is kept\n{}",
        case.lock()
    );

    case.ok(
        "lock --upgrade-package zzz",
        &["lock", "--index", new.as_str(), "--upgrade-package", "zzz"],
    );
    assert!(
        lock_block(&case.lock(), "a").contains("version = \"1.0\""),
        "freeing another name leaves a alone\n{}",
        case.lock()
    );

    case.ok(
        "lock --upgrade-package a",
        &["lock", "--index", new.as_str(), "--upgrade-package", "a"],
    );
    assert!(
        lock_block(&case.lock(), "a").contains("version = \"2.0\""),
        "--upgrade-package a frees a\n{}",
        case.lock()
    );

    case.ok("lock (back to 1.0)", &["lock", "--index", old.as_str()]);
    assert!(lock_block(&case.lock(), "a").contains("version = \"1.0\""));
    case.ok(
        "lock --upgrade",
        &["lock", "--index", new.as_str(), "--upgrade"],
    );
    assert!(
        lock_block(&case.lock(), "a").contains("version = \"2.0\""),
        "--upgrade frees every pin\n{}",
        case.lock()
    );
    case.ok(
        "lock --check",
        &["lock", "--index", new.as_str(), "--check"],
    );
}

#[test]
fn add_reads_a_requirements_file_and_remove_takes_names_and_scopes() {
    let case = Case::start();
    let reqs = case.project.join("requirements.txt");
    write(
        &reqs,
        "# pinned by hand\na==1.0  # trailing comment\nb==1.0\n",
    );
    case.add("add -r requirements.txt", &["-r", "requirements.txt"]);
    let manifest = read(&case.manifest());
    assert!(
        manifest.contains("dependencies = [\n    \"a==1.0\",\n    \"b==1.0\",\n]"),
        "-r must add every requirement in the file\n{manifest}"
    );

    let args = ["add", "--index", "unused"];
    let out = case.run(&args);
    let said = expect_failure("add with nothing to add", &args, &out);
    assert!(said.contains("nothing to add"), "{}", render(&args, &out));

    case.add("add --dev b", &["--dev", "b"]);
    case.ok("remove --dev b", &["remove", "--dev", "b"]);
    let manifest = read(&case.manifest());
    assert!(
        manifest.contains("dev = []") && manifest.contains("\"b==1.0\""),
        "remove --dev must touch only the dev group\n{manifest}"
    );

    case.ok("remove a b", &["remove", "a", "b"]);
    let manifest = read(&case.manifest());
    assert!(
        manifest.contains("dependencies = []"),
        "remove takes several names\n{manifest}"
    );
    assert!(
        !case.lock().contains("[[package]]"),
        "the lock follows the manifest\n{}",
        case.lock()
    );
}

#[test]
fn run_no_project_skips_the_preflight_and_python_overrides_the_interpreter() {
    let case = Case::start();
    case.add("add a", &["a"]);
    let script = case.project.join("probe.py");
    write(
        &script,
        "import os, sys\nprint(sys.executable)\nprint(os.environ.get('VIRTUAL_ENV', '-'))\n",
    );

    // Locked but never synced: the project preflight refuses.
    let args = ["run", "probe.py"];
    let out = case.run(&args);
    let said = expect_failure("run before sync", &args, &out);
    assert!(said.contains("mamba sync"), "{}", render(&args, &out));

    // `--no-project` runs the file as if no project were here.
    let out = case.ok(
        "run --no-project before sync",
        &["run", "--no-project", "probe.py"],
    );
    let lines: Vec<String> = stdout_of(&out).lines().map(str::to_string).collect();
    assert_eq!(lines.len(), 2, "{}", render(&args, &out));
    assert!(
        !lines[0].contains(".venv"),
        "--no-project must not run on the project environment: {}",
        lines[0]
    );
    assert_eq!(lines[1], "-", "--no-project injects no VIRTUAL_ENV");

    case.with_venv();
    case.ok("sync", &["sync"]);
    let out = case.ok("run (project)", &["run", "probe.py"]);
    let lines: Vec<String> = stdout_of(&out).lines().map(str::to_string).collect();
    // `sys.executable` is the venv's own `bin/python` symlink, so compare
    // the path as printed rather than the interpreter it links to.
    let venv_bin = canon(&case.project.join(".venv")).join("bin");
    assert!(
        Path::new(&lines[0]).starts_with(&venv_bin)
            || lines[0].contains("/.venv/bin/")
            || lines[0].contains("\\.venv\\Scripts\\"),
        "a synced project runs on its own interpreter: {}",
        lines[0]
    );
    assert_eq!(
        canon(Path::new(&lines[1])),
        canon(&case.project.join(".venv"))
    );

    let python = path_arg(&case.python3);
    let out = case.ok(
        "run --python",
        &["run", "--python", python.as_str(), "probe.py"],
    );
    let lines: Vec<String> = stdout_of(&out).lines().map(str::to_string).collect();
    assert_eq!(
        canon(Path::new(&lines[0])),
        canon(&case.python3),
        "--python runs the file on the named interpreter"
    );

    let args = ["run", "--python", "./no-such-python", "probe.py"];
    let out = case.run(&args);
    let said = expect_failure("run --python missing", &args, &out);
    assert!(said.contains("no-such-python"), "{}", render(&args, &out));
}
