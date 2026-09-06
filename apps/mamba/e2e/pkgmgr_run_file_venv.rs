//! Black-box contract: `mamba run <file>` must execute the file on the
//! project's `.venv` interpreter, and `--compile` must be the explicit opt-in
//! that hands it to the Mamba compiler instead.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary over a throwaway directory tree
//! and reads only what the binary wrote, what it printed, or what the
//! project's own interpreter reports about itself. Nothing is hand-placed in
//! the environment: the distribution is a real wheel built by the product's
//! own `WheelBuilder`, frozen by `mamba index build`, locked by `mamba add`,
//! and installed by `mamba sync`, so `import lib` succeeding means those four
//! steps and the interpreter `run` chose all agree on one artifact.
//!
//! # The observation point
//!
//! The script under test prints `sys.executable`, and that string is the whole
//! oracle for "which interpreter ran this file". It is never compared as text
//! against a path the case built by hand:
//!
//! - **Inside the project**, the assertion is that the *grandparent directory*
//!   of the reported executable canonicalises to `canonicalize(<project>/.venv)`.
//!   The reported path is the un-resolved launch path — `<project>/.venv/bin/python`
//!   is a symlink to the base interpreter, so canonicalising the executable
//!   itself would resolve straight out of the environment and prove the
//!   opposite of what is being asserted. Canonicalising the two *directories*
//!   is also what makes the comparison survive macOS reporting `/var/folders/…`
//!   as `/private/var/folders/…`.
//! - **Outside a project**, the expected string is produced by running the
//!   `python3` the case found on `PATH` — the launcher itself, not a resolution
//!   of it — under the same environment the `mamba` child gets, and asking it
//!   the same question. On a host where `python3` is a version-manager shim
//!   both sides therefore report the same real interpreter, and the shim's
//!   indirection cannot make the case pass or fail on its own.
//!
//! The second and third lines of the same stdout close the two remaining ways
//! a wrong interpreter could still look right: `lib.answer()` returning `42`
//! can only come from the wheel's own bytes installed in the environment, and
//! `sys.argv[1:]` being `['extra']` pins the frozen decision that arguments
//! after the file belong to the script.
//!
//! # Why today's tree cannot pass
//!
//! `cmd_run` (`apps/mamba/src/main.rs`) enters command mode only for
//! `run -- <cmd>`; every other file goes to `CompilerSession::run`. The `run`
//! verb declares no `--compile` flag and no positional after `<file>`, so
//! three of the four observations below are refused before the file is even
//! opened, and the fourth is answered by the compiler:
//!
//! 1. `mamba run main.py extra` — clap: `error: unexpected argument 'extra' found`, exit 2.
//! 2. `mamba run exit3.py` — the compiler turns `sys.exit(3)` into
//!    `error: SystemExit: 3` and exits 1, so the script's own code is lost.
//! 3. `mamba run --compile hello.py` — clap: `error: unexpected argument '--compile' found`, exit 2.
//! 4. `mamba run main.py` outside a project — exit 0, but `sys.executable` is
//!    the `mamba` binary itself, not any Python interpreter.
//!
//! # Facets
//!
//! - **Behavior**: the default route for a file is the project's own
//!   interpreter with the project's own installed packages; the script's exit
//!   code survives; `--compile` still reaches the compiler; and a file outside
//!   a project runs on the `PATH` interpreter without inventing a project.
//!   The exit-code case is the failure mode that a wrapper is most likely to
//!   swallow, so it is asserted as an exact code rather than as
//!   "non-zero" — `Some(3)`, not `!= Some(0)`.
//! - **Security (the environment is the authority for what executes)**: an
//!   interpreter chosen from anywhere but the synced project is the fail-open
//!   state this work item closes, so the case clears `VIRTUAL_ENV` from every
//!   child it spawns — a `run` that merely inherited an ambient activation
//!   would find nothing to inherit here — and removes `PYTHONPATH`, so the
//!   only way `import lib` can succeed is through the environment's own
//!   `site`. The out-of-project half asserts the containment side: no
//!   `environment is not synced` refusal is emitted where there is no project
//!   to be out of sync with, so widening the venv route must not start
//!   gating unrelated directories.
//! - **Performance**: the work item names no budget, so this case asserts no
//!   timing.
//!
//! # Hermeticity
//!
//! No network, ever: the wheel is built in-process and the index is a local
//! directory. `HOME` and `MAMBA_CACHE_DIR` are pinned into the temp tree;
//! `MAMBA_FROZEN_INDEX`, `MAMBA_INDEX_URL`, `MAMBA_JOBS`, `XDG_CACHE_HOME`,
//! `VIRTUAL_ENV`, and `PYTHONPATH` are removed so no ambient value can supply
//! an answer. Every invocation runs inside a `tempfile::tempdir()` that stays
//! alive for the whole test. Nothing is skipped and nothing is `#[ignore]`d:
//! a host with no `python3` fails the case naming it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mamba::pkgmanage::pkgmgr::wheel_build::{
    compose_filename, CoreMetadata, WheelBuilder, WheelMetadata,
};

/// The distribution the project depends on: one module, one function.
const DIST: &str = "lib";
const VERSION: &str = "1.0";
const MODULE: &str = "lib";
const WHEEL_FILE: &str = "lib-1.0-py3-none-any.whl";
/// The module body the wheel carries. `answer()` exists only in the packaged
/// source, so reading `42` back proves the interpreter that ran the file saw
/// the environment the project installed into.
const MODULE_BODY: &str = "def answer():\n    return 42\n";
/// What `lib.answer()` prints once the wheel is importable.
const ANSWER: &str = "42";

/// Three lines: which interpreter is running, whether the project's packages
/// are importable from it, and what the script was handed as its arguments.
const MAIN_PY: &str = "import sys, lib\n\
                       print(sys.executable)\n\
                       print(lib.answer())\n\
                       print(sys.argv[1:])\n";
/// One line: which interpreter is running. No project, so no import.
const BARE_MAIN_PY: &str = "import sys\nprint(sys.executable)\n";
/// A script whose only observable is the code it exits with.
const EXIT3_PY: &str = "import sys\nsys.exit(3)\n";
/// The compiler's half: a program with no imports and one printed word.
const HELLO_PY: &str = "print(\"hello\")\n";

/// The trailing argument `run` must hand to the script rather than consume.
const SCRIPT_ARG: &str = "extra";
/// How Python renders `sys.argv[1:]` when it was handed exactly that argument.
const EXPECTED_ARGV: &str = "['extra']";
/// The compiler half's expected stdout.
const EXPECTED_HELLO: &str = "hello";
/// The preflight refusal that belongs to a project with an unsynced venv, and
/// must not be emitted where there is no project at all.
const NOT_SYNCED: &str = "environment is not synced";

/// The binary under test. `CARGO_BIN_EXE_*` is resolved by cargo at compile
/// time, so a missing binary is a build failure rather than a skipped case.
fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

/// Run the binary in `cwd` with an environment that cannot reach a registry,
/// a user-global cache, an ambient activation, or an ambient import path.
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

/// Require exit 0, reporting the whole invocation when it is not.
fn expect_ok(step: &str, args: &[&str], out: &Output) {
    assert!(
        out.status.success(),
        "step `{step}` must succeed: {}",
        render(args, out)
    );
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn canon(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn path_arg(path: &Path) -> String {
    path.to_str()
        .unwrap_or_else(|| panic!("fixture: path {} is not utf-8", path.display()))
        .to_string()
}

fn write_script(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, body).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    path
}

/// The `python3` this host offers, in the two forms this case needs.
struct Python {
    /// The first `python3` on `PATH`, exactly as `mamba` will find it. Asking
    /// *this* for `sys.executable` is the out-of-project oracle, so a
    /// version-manager shim answers for both sides of that comparison.
    launcher: PathBuf,
    /// What the launcher reports as `sys.executable`: the real interpreter,
    /// which `mamba venv --python` can seed an environment from under a
    /// pinned `HOME` that the shim's own resolution might otherwise depend on.
    real: PathBuf,
}

/// Resolve `python3` on `PATH`. A host with none fails here, naming it — the
/// case never skips and is never `#[ignore]`d.
fn resolve_python3() -> Python {
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
    let launcher = candidate.unwrap_or_else(|| {
        panic!(
            "this case needs `python3` on PATH and found none in {:?}",
            path_var
        )
    });
    let out = Command::new(&launcher)
        .args(["-c", "import sys; print(sys.executable)"])
        .output()
        .unwrap_or_else(|e| panic!("spawn {}: {e}", launcher.display()));
    assert!(
        out.status.success(),
        "`{} -c 'print(sys.executable)'` failed: {}",
        launcher.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    let real = PathBuf::from(stdout_of(&out).trim());
    assert!(
        real.is_file(),
        "`{}` reported the interpreter {}, which is not a file",
        launcher.display(),
        real.display()
    );
    Python { launcher, real }
}

/// The directory two levels above `executable` — `<venv>` for
/// `<venv>/bin/python` and for `<venv>\Scripts\python.exe` alike.
fn environment_root_of(executable: &str) -> PathBuf {
    let path = Path::new(executable);
    path.parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| {
            panic!(
                "`mamba run` reported the interpreter `{executable}`, which has no \
                 enclosing environment directory (expected `<env>/bin/python` or \
                 `<env>\\Scripts\\python.exe`)"
            )
        })
        .to_path_buf()
}

/// One scaffolded project: `mamba init`, a real PEP 405 environment seeded
/// from the host `python3`, and `lib==1.0` locked from a frozen local index
/// and installed by `mamba sync`.
struct Case {
    _root: tempfile::TempDir,
    project: PathBuf,
    home: PathBuf,
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
            project.join("pyproject.toml").is_file(),
            "fixture: `mamba init` wrote no pyproject.toml in {}",
            project.display()
        );

        let python = resolve_python3();
        let python_arg = path_arg(&python.real);
        let venv_args = ["venv", "--python", python_arg.as_str()];
        let out = run(&project, &home, &venv_args);
        expect_ok("venv --python <python3>", &venv_args, &out);

        let index = root.join("index");
        let wheel = build_wheel(&root.join("wheels"));
        let index_arg = path_arg(&index);
        let wheel_arg = path_arg(&wheel);
        let build_args = [
            "index",
            "build",
            "--out",
            index_arg.as_str(),
            wheel_arg.as_str(),
        ];
        let out = run(&project, &home, &build_args);
        expect_ok("index build", &build_args, &out);

        let add_args = ["add", DIST, "--index", index_arg.as_str()];
        let out = run(&project, &home, &add_args);
        expect_ok("add --index", &add_args, &out);

        let sync_args = ["sync"];
        let out = run(&project, &home, &sync_args);
        expect_ok("sync", &sync_args, &out);

        let case = Case {
            _root: root_dir,
            project,
            home,
        };
        case.assert_environment_imports_the_wheel();
        case
    }

    /// `.venv/bin/python` (POSIX) or `.venv/Scripts/python.exe` (Windows), as
    /// `python -m venv` laid it down.
    fn venv_python(&self) -> PathBuf {
        let venv = self.project.join(".venv");
        for candidate in [
            venv.join("bin").join("python"),
            venv.join("Scripts").join("python.exe"),
        ] {
            if candidate.is_file() {
                return candidate;
            }
        }
        panic!(
            "fixture: `mamba venv` left no interpreter under {}",
            venv.display()
        );
    }

    /// Fixture self-check, asked of the environment's own interpreter and not
    /// of `mamba run`: the wheel really is importable from `.venv`. A red here
    /// is a broken fixture — `init`, `venv`, `index build`, `add`, or `sync` —
    /// never the routing this case is about.
    fn assert_environment_imports_the_wheel(&self) {
        let python = self.venv_python();
        let out = Command::new(&python)
            .args(["-c", "import lib; print(lib.answer())"])
            .current_dir(&self.project)
            .env("HOME", &self.home)
            .env_remove("PYTHONPATH")
            .env_remove("VIRTUAL_ENV")
            .output()
            .unwrap_or_else(|e| panic!("spawn {}: {e}", python.display()));
        assert!(
            out.status.success() && stdout_of(&out).trim() == ANSWER,
            "fixture: the synced environment must import {DIST}=={VERSION}; \
             `{} -c 'import lib; print(lib.answer())'` exited {:?}\n\
             --- stdout ---\n{}\n--- stderr ---\n{}",
            python.display(),
            out.status.code(),
            stdout_of(&out),
            stderr_of(&out),
        );
    }

    fn run(&self, args: &[&str]) -> Output {
        run(&self.project, &self.home, args)
    }

    fn write(&self, name: &str, body: &str) -> PathBuf {
        write_script(&self.project, name, body)
    }

    /// `canonicalize(<project>/.venv)` — the one directory a file run inside
    /// this project may report an interpreter from.
    fn venv_dir(&self) -> PathBuf {
        canon(&self.project.join(".venv"))
    }
}

/// Build the real, importable wheel through the product's own wheel builder.
fn build_wheel(out_dir: &Path) -> PathBuf {
    let filename = compose_filename(DIST, VERSION, "py3", "none", "any");
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-run-file-venv");
    wheel_meta.tags.push("py3-none-any".into());
    let core_meta = CoreMetadata::new(DIST, VERSION);
    let mut builder = WheelBuilder::new(filename, wheel_meta, core_meta);
    builder.add_file(format!("{MODULE}/__init__.py"), MODULE_BODY.to_string());
    let wheel = builder
        .build_to_dir(out_dir)
        .unwrap_or_else(|e| panic!("fixture: build wheel {DIST}-{VERSION}: {e:?}"));
    assert_eq!(
        wheel.file_name().and_then(|n| n.to_str()),
        Some(WHEEL_FILE),
        "fixture: the wheel builder named the artifact {}",
        wheel.display()
    );
    wheel
}

#[test]
fn run_file_executes_on_the_project_venv_interpreter() {
    let case = Case::start();
    case.write("main.py", MAIN_PY);

    let args = ["run", "main.py", SCRIPT_ARG];
    let out = case.run(&args);
    expect_ok("run main.py extra", &args, &out);

    let stdout = stdout_of(&out);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        3,
        "`mamba run main.py {SCRIPT_ARG}` must pass the script's stdout through \
         unchanged — three lines, nothing added: {}",
        render(&args, &out)
    );

    // 1. The interpreter is the project's own, compared as canonical
    //    directories: `.venv/bin/python` is a symlink to the base interpreter,
    //    so the reported path itself must not be resolved.
    let reported = lines[0];
    let environment = canon(&environment_root_of(reported));
    let expected = case.venv_dir();
    assert_eq!(
        environment,
        expected,
        "`mamba run <file>` must execute the file on the project's own \
         interpreter: it reported `{reported}`, whose environment directory is \
         {}, not {}. {}",
        environment.display(),
        expected.display(),
        render(&args, &out)
    );

    // 2. That interpreter sees the project's installed packages.
    assert_eq!(
        lines[1],
        ANSWER,
        "the interpreter running the file must import the project's locked \
         {DIST}=={VERSION}: {}",
        render(&args, &out)
    );

    // 3. Arguments after the file belong to the script, not to `mamba run`.
    assert_eq!(
        lines[2],
        EXPECTED_ARGV,
        "arguments after the file must be passed to the script as \
         `sys.argv[1:]`: {}",
        render(&args, &out)
    );
}

#[test]
fn run_file_propagates_the_script_exit_code() {
    let case = Case::start();
    case.write("exit3.py", EXIT3_PY);

    let args = ["run", "exit3.py"];
    let out = case.run(&args);
    assert_eq!(
        out.status.code(),
        Some(3),
        "`mamba run <file>` must exit with the script's own exit code, not \
         rewrite it: {}",
        render(&args, &out)
    );
}

#[test]
fn run_compile_flag_routes_the_file_to_the_compiler() {
    let case = Case::start();
    case.write("hello.py", HELLO_PY);

    let args = ["run", "--compile", "hello.py"];
    let out = case.run(&args);
    expect_ok("run --compile hello.py", &args, &out);
    assert_eq!(
        stdout_of(&out).trim(),
        EXPECTED_HELLO,
        "`mamba run --compile <file>` must be the explicit opt-in that reaches \
         the compiler, and the compiler's output must be unchanged: {}",
        render(&args, &out)
    );
}

#[test]
fn run_file_outside_a_project_uses_the_path_interpreter() {
    let root_dir = tempfile::tempdir().expect("create temp root");
    let root = root_dir.path().to_path_buf();
    let loose = root.join("loose");
    let home = root.join("home");
    for dir in [&loose, &home] {
        std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("create {}: {e}", dir.display()));
    }
    assert!(
        !loose.join("pyproject.toml").exists(),
        "fixture: {} must hold no pyproject.toml",
        loose.join("pyproject.toml").display()
    );

    let python = resolve_python3();
    write_script(&loose, "main.py", BARE_MAIN_PY);

    // The oracle: the same launcher `mamba` will find on `PATH`, asked the
    // same question under the same environment. On a host where `python3` is a
    // version-manager shim, both sides report the real interpreter it execs,
    // so the shim cannot decide this assertion.
    let probe = Command::new(&python.launcher)
        .args(["-c", "import sys; print(sys.executable)"])
        .current_dir(&loose)
        .env("HOME", &home)
        .env_remove("VIRTUAL_ENV")
        .env_remove("PYTHONPATH")
        .output()
        .unwrap_or_else(|e| panic!("spawn {}: {e}", python.launcher.display()));
    assert!(
        probe.status.success(),
        "fixture: `{} -c 'print(sys.executable)'` failed: {}",
        python.launcher.display(),
        stderr_of(&probe)
    );
    let expected = stdout_of(&probe).trim().to_string();
    assert!(
        Path::new(&expected).is_file(),
        "fixture: `{}` reported the interpreter `{expected}`, which is not a file",
        python.launcher.display()
    );

    let args = ["run", "main.py"];
    let out = run(&loose, &home, &args);
    expect_ok("run main.py (no project)", &args, &out);
    assert_eq!(
        stdout_of(&out).trim(),
        expected,
        "outside a project `mamba run <file>` must execute the file on the \
         `python3` found on PATH — the interpreter `{}` reports as \
         `{expected}`. {}",
        python.launcher.display(),
        render(&args, &out)
    );
    assert!(
        !stderr_of(&out).contains(NOT_SYNCED),
        "there is no project in {}, so `mamba run <file>` must not refuse with \
         `{NOT_SYNCED}`: {}",
        loose.display(),
        render(&args, &out)
    );
}
