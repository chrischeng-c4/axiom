//! Black-box contract: with a project environment present, `mamba pip install`
//! and `mamba pip sync` install into *that* environment's interpreter without
//! being told which interpreter it is.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary over a throwaway tree and reads
//! only what the binary wrote or what the project's own interpreter reports.
//! The distribution is built in-process by the product's own `WheelBuilder`
//! with an `entry_points.txt` carrying one `[console_scripts]` entry, frozen
//! by `mamba index build`, and installed by `mamba pip install` / `mamba pip
//! sync` against a project whose `.venv` was created by `mamba venv --python
//! <absolute python3>`. Nothing installed is hand-written, and no command in
//! the case names an interpreter except where naming it is the point.
//!
//! # The observation point
//!
//! Three separate things say "it went into the project's environment", and a
//! passing run needs all three:
//!
//! 1. the dist-info sits in the directory the environment's own interpreter
//!    reports as `purelib` (`sysconfig.get_paths()`), never a path this case
//!    computed from `.venv` plus literals;
//! 2. the console-script wrapper's first line is `#!` followed by the absolute
//!    path of the environment's interpreter — compared as a *name*, not as a
//!    resolved target, because `.venv/bin/python` is itself a symlink to the
//!    interpreter that seeded the environment and resolving it would erase the
//!    difference this case exists to observe; and
//! 3. that wrapper, executed by its own absolute path with `PATH` removed from
//!    the child environment, exits 0 printing the entry point's own line, and
//!    the environment's interpreter imports the installed module.
//!
//! # Why the current tree cannot pass
//!
//! `install_options` (`src/pkgmanage/pip.rs`) defaults `--site-packages` from
//! the project's `.venv` but defaults `--python` to the *relative* literal
//! `python3`. `Installer::install` refuses a non-absolute interpreter before
//! writing anything whenever the wheel declares console scripts, so today the
//! two default-driven cases below exit 1 with
//! `Error: io error at python3: python_executable must be an absolute path to
//! derive the console-script directory; got python3` and install nothing —
//! while the two cases that pin what must *not* change (the explicit
//! `--python` flag, and the fail-closed refusal when no environment exists)
//! already hold.
//!
//! # Facets
//!
//! - **Behavior**: for both entry points a user reaches this through —
//!   `mamba pip install <req>` and `mamba pip sync <requirements.txt>` — the
//!   command exits 0, says `installed cliapp==1.0`, leaves the dist-info where
//!   the environment imports from, and leaves a runnable console script beside
//!   the environment's interpreter.
//! - **Security (what an install may reach, and how it fails)**: a console
//!   script is an executable file this tool writes and the user then runs, so
//!   the interpreter its shebang names is a boundary, not a detail. The case
//!   pins containment — the shebang is exactly the project's own interpreter,
//!   so a wrapper cannot silently import a host environment's packages instead
//!   of the ones the project pinned — and it pins that an explicit `--python`
//!   still wins over the derived default, so the default can never override a
//!   caller's stated environment. It also pins the fail-closed path: with no
//!   `.venv` to derive from, the command must still refuse, name the reason,
//!   install nothing, and not panic. A default that reached PATH, `which`, or
//!   the compiler's own interpreter would turn that refusal into a silent
//!   install against an arbitrary interpreter, so "some absolute path" is not
//!   enough.
//! - **Performance**: the work item names no budget, so this case asserts no
//!   timing.
//!
//! # Hermeticity
//!
//! No network, ever: the wheel is built in-process and the index is a local
//! directory. `HOME` and `MAMBA_CACHE_DIR` are pinned into the temp tree;
//! `MAMBA_FROZEN_INDEX`, `MAMBA_INDEX_URL`, `MAMBA_JOBS`, `XDG_CACHE_HOME`,
//! `VIRTUAL_ENV`, and `PYTHONPATH` are removed so no ambient value can supply
//! the answer the product is supposed to derive. Nothing is skipped and
//! nothing is `#[ignore]`d: a missing `python3` fails the case naming it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mamba::pkgmanage::pkgmgr::wheel_build::{
    compose_filename, CoreMetadata, WheelBuilder, WheelMetadata,
};

/// The distribution under test: one importable package, one entry point.
const DIST: &str = "cliapp";
const VERSION: &str = "1.0";
const MODULE: &str = "cliapp";
const WHEEL_FILE: &str = "cliapp-1.0-py3-none-any.whl";
const DIST_INFO: &str = "cliapp-1.0.dist-info";

/// The console script's name. Deliberately unlike `DIST` and `MODULE` so a
/// sweep for the wrapper cannot collide with the package directory or its
/// dist-info and report a false find.
const SCRIPT: &str = "sentinelctl";

/// The line the entry point prints and nothing else does, so reading it back
/// off a child process's stdout proves the wheel's own function ran.
const SENTINEL: &str = "sentinel ok";

/// The action line `mamba pip install` / `mamba pip sync` print per install.
const INSTALLED_LINE: &str = "installed cliapp==1.0";

/// The installer's fail-closed reason when it cannot locate the interpreter a
/// console script's shebang must name. Pinned here because the no-environment
/// case must keep refusing for exactly this reason after the default changes.
const ABSOLUTE_PATH_REFUSAL: &str =
    "python_executable must be an absolute path to derive the console-script directory";

/// The module the entry point names: `main` prints the sentinel and returns 0,
/// which the wrapper's `sys.exit(main())` turns into exit status 0.
const CLI_BODY: &str = r#"def main():
    print("sentinel ok")
    return 0
"#;

/// One `[console_scripts]` entry, in the `entry_points.txt` shape PEP 517
/// backends emit.
const ENTRY_POINTS_TXT: &str = "[console_scripts]\nsentinelctl = cliapp.cli:main\n";

/// The binary under test. `CARGO_BIN_EXE_*` is resolved by cargo at compile
/// time, so a missing binary is a build failure rather than a skipped case.
fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

/// Run the binary in `cwd` with an environment that cannot reach a registry, a
/// user-global cache, or an ambient import path.
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

/// Absolute and comparable, resolved *by directory only*: the parent is
/// canonicalized (macOS spells one temp directory both `/var/folders/...` and
/// `/private/var/folders/...`, and a child process reports the second), while
/// the final component is kept exactly as written. Resolving the whole path
/// would follow `.venv/bin/python` out to the interpreter that seeded the
/// environment, making two different names for one interpreter compare equal —
/// and telling those two names apart is what
/// `the_explicit_python_flag_still_wins_over_the_derived_default` measures.
fn named_abs(path: &Path) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    match path.file_name() {
        Some(name) => canon(parent).join(name),
        None => canon(path),
    }
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

/// Every path under `dir` whose file name is `name`, symlinked directories not
/// descended into (a venv's `lib64` is commonly a symlink to `lib`).
fn find_named(dir: &Path, name: &str, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.file_name().and_then(|n| n.to_str()) == Some(name) {
            found.push(path.clone());
        }
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.is_dir() {
            find_named(&path, name, found);
        }
    }
}

/// Build the real, importable wheel through the product's own wheel builder,
/// including the `entry_points.txt` body that declares the console script.
fn build_wheel(out_dir: &Path) -> PathBuf {
    let filename = compose_filename(DIST, VERSION, "py3", "none", "any");
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-pip-default-interpreter");
    wheel_meta.tags.push("py3-none-any".into());
    let core_meta = CoreMetadata::new(DIST, VERSION);
    let mut builder = WheelBuilder::new(filename, wheel_meta, core_meta);
    builder.add_file(format!("{MODULE}/__init__.py"), String::new());
    builder.add_file(format!("{MODULE}/cli.py"), CLI_BODY.to_string());
    builder.set_entry_points(ENTRY_POINTS_TXT.to_string());
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

/// One throwaway tree: an isolated `HOME`, a frozen index holding the
/// console-script wheel, and one project directory. `with_venv` is the single
/// premise the four cases differ on — whether that project has an environment
/// for the product to derive an interpreter from.
struct Case {
    _root: tempfile::TempDir,
    root: PathBuf,
    project: PathBuf,
    home: PathBuf,
    index: PathBuf,
}

impl Case {
    fn scaffold(with_venv: bool) -> Case {
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

        if with_venv {
            let python3 = resolve_python3();
            let python_arg = path_arg(&python3);
            let venv_args = ["venv", "--python", python_arg.as_str()];
            let out = run(&project, &home, &venv_args);
            expect_ok("venv --python <python3>", &venv_args, &out);
            assert!(
                project.join(".venv").join("pyvenv.cfg").is_file(),
                "fixture: `mamba venv` left no pyvenv.cfg under {}",
                project.join(".venv").display()
            );
        } else {
            assert!(
                !project.join(".venv").exists(),
                "fixture: this case needs a project with no environment, and {} exists",
                project.join(".venv").display()
            );
        }

        Case {
            _root: root_dir,
            root,
            project,
            home,
            index,
        }
    }

    fn with_venv() -> Case {
        Case::scaffold(true)
    }

    fn without_venv() -> Case {
        Case::scaffold(false)
    }

    fn index_arg(&self) -> String {
        path_arg(&self.index)
    }

    fn run(&self, args: &[&str]) -> Output {
        run(&self.project, &self.home, args)
    }

    fn venv(&self) -> PathBuf {
        self.project.join(".venv")
    }

    /// `.venv/bin/python` (POSIX) or `.venv/Scripts/python.exe` (Windows), as
    /// `python -m venv` laid it down.
    fn venv_python(&self) -> PathBuf {
        let venv = self.venv();
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

    /// The directory holding that interpreter — where a console script belongs,
    /// because it lives beside the interpreter its own shebang names.
    fn venv_bin(&self) -> PathBuf {
        self.venv_python()
            .parent()
            .expect("fixture: the venv interpreter has a parent directory")
            .to_path_buf()
    }

    /// Ask the environment's own interpreter where installed packages are
    /// imported from. This is the case's only definition of "the venv's
    /// site-packages": it is never `.venv` plus literals.
    fn purelib(&self) -> PathBuf {
        let out = self.python_probe(
            &self.venv_python(),
            "import sysconfig; print(sysconfig.get_paths()['purelib'])",
        );
        assert!(
            out.status.success(),
            "fixture: asking {} for its purelib failed: {}",
            self.venv_python().display(),
            stderr_of(&out)
        );
        let purelib = PathBuf::from(stdout_of(&out).trim());
        assert!(
            purelib.is_dir(),
            "fixture: the environment reported the import directory {}, which is not a directory",
            purelib.display()
        );
        assert!(
            canon(&purelib).starts_with(canon(&self.venv())),
            "fixture: the environment reported the import directory {}, which is outside {}",
            purelib.display(),
            self.venv().display()
        );
        purelib
    }

    /// Run `<python> -c <code>` with no ambient import path.
    fn python_probe(&self, python: &Path, code: &str) -> Output {
        Command::new(python)
            .args(["-c", code])
            .current_dir(&self.project)
            .env("HOME", &self.home)
            .env_remove("PYTHONPATH")
            .env_remove("VIRTUAL_ENV")
            .output()
            .unwrap_or_else(|e| panic!("spawn {}: {e}", python.display()))
    }

    /// A second absolute name for a working interpreter, inside the tree and
    /// outside the environment. Used to prove an explicit `--python` still
    /// decides where the wrapper goes and which interpreter it names.
    fn alternate_interpreter(&self) -> PathBuf {
        let dir = self.root.join("alt-prefix");
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("create {}: {e}", dir.display()));
        let alt = dir.join("python-alt");
        let target = self.venv_python();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, &alt).unwrap_or_else(|e| {
            panic!(
                "fixture: link {} -> {}: {e}",
                alt.display(),
                target.display()
            )
        });
        #[cfg(not(unix))]
        {
            std::fs::copy(&target, &alt).unwrap_or_else(|e| {
                panic!(
                    "fixture: copy {} -> {}: {e}",
                    target.display(),
                    alt.display()
                )
            });
        }
        assert!(
            alt.exists(),
            "fixture: the alternate interpreter {} was not created",
            alt.display()
        );
        alt
    }

    /// Read the first line of a console-script wrapper, requiring a shebang.
    fn shebang_of(&self, wrapper: &Path, origin: &str) -> String {
        let body = std::fs::read_to_string(wrapper)
            .unwrap_or_else(|e| panic!("[{origin}] read {}: {e}", wrapper.display()));
        body.lines()
            .next()
            .unwrap_or_default()
            .strip_prefix("#!")
            .unwrap_or_else(|| {
                panic!(
                    "[{origin}] the console script {} must start with a shebang\
                     \n--- the wrapper's own bytes ---\n{body}",
                    wrapper.display()
                )
            })
            .trim()
            .to_string()
    }

    /// Execute the wrapper file itself: absolute path, no shell, and no `PATH`
    /// in the child's environment, so nothing but the file at `path` can
    /// answer.
    fn assert_wrapper_runs(&self, wrapper: &Path, origin: &str) {
        let out = Command::new(wrapper)
            .current_dir(&self.project)
            .env("HOME", &self.home)
            .env_remove("PATH")
            .env_remove("PYTHONPATH")
            .env_remove("PYTHONHOME")
            .env_remove("VIRTUAL_ENV")
            .output()
            .unwrap_or_else(|e| panic!("[{origin}] spawn the wrapper {}: {e}", wrapper.display()));
        assert!(
            out.status.success(),
            "[{origin}] running the console script {} must exit 0; exited {:?}\
             \n--- stdout ---\n{}\n--- stderr ---\n{}\n--- the wrapper's own bytes ---\n{}",
            wrapper.display(),
            out.status.code(),
            stdout_of(&out),
            stderr_of(&out),
            std::fs::read_to_string(wrapper).unwrap_or_else(|e| format!("<unreadable: {e}>")),
        );
        assert_eq!(
            stdout_of(&out).trim(),
            SENTINEL,
            "[{origin}] the console script must run the wheel's own entry point, which prints \
             `{SENTINEL}`; it printed {:?} on stdout and {:?} on stderr",
            stdout_of(&out),
            stderr_of(&out),
        );
    }

    /// Everything the work item promises about one default-driven install.
    /// `origin` names the entry point so a failure says which one broke.
    fn assert_installed_into_the_project_environment(
        &self,
        origin: &str,
        args: &[&str],
        out: &Output,
    ) {
        expect_ok(origin, args, out);
        assert!(
            stdout_of(out).lines().any(|l| l.trim() == INSTALLED_LINE),
            "[{origin}] the command must report `{INSTALLED_LINE}` on stdout: {}",
            render(args, out)
        );

        // 1. The distribution landed where this environment imports from.
        let purelib = self.purelib();
        let dist_info = purelib.join(DIST_INFO);
        assert!(
            dist_info.is_dir(),
            "[{origin}] the install must place {DIST_INFO} in the directory the environment \
             itself imports from, {}: it is missing.\n  present there: {:?}\n{}",
            purelib.display(),
            listing(&purelib),
            render(args, out)
        );

        // 2. The console script landed beside the environment's interpreter,
        //    and its shebang names exactly that interpreter.
        let wrapper = self.venv_bin().join(SCRIPT);
        assert!(
            wrapper.is_file(),
            "[{origin}] the install must write the console script `{SCRIPT}` beside the \
             environment's interpreter, in {}: {} is missing.\n  present there: {:?}\n{}",
            self.venv_bin().display(),
            wrapper.display(),
            listing(&self.venv_bin()),
            render(args, out)
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&wrapper)
                .unwrap_or_else(|e| panic!("[{origin}] stat {}: {e}", wrapper.display()))
                .permissions()
                .mode();
            assert!(
                mode & 0o111 != 0,
                "[{origin}] the console script {} must be executable; its mode is {mode:o}",
                wrapper.display(),
            );
        }
        let shebang = self.shebang_of(&wrapper, origin);
        assert_eq!(
            named_abs(Path::new(&shebang)),
            named_abs(&self.venv_python()),
            "[{origin}] the console script's shebang must name the project environment's own \
             interpreter, {}, and nothing else; it names {shebang}",
            self.venv_python().display(),
        );

        // 3. The wrapper runs, and the module imports through that interpreter.
        self.assert_wrapper_runs(&wrapper, origin);
        let import = self.python_probe(&self.venv_python(), &format!("import {MODULE}"));
        assert!(
            import.status.success(),
            "[{origin}] the environment's interpreter {} must import the installed module \
             `{MODULE}`; exited {:?}\n--- stderr ---\n{}",
            self.venv_python().display(),
            import.status.code(),
            stderr_of(&import),
        );
    }
}

/// The work item's Goal: no `--python`, and the install lands in the project's
/// own environment.
#[test]
fn pip_install_without_python_installs_into_the_project_environment() {
    let case = Case::with_venv();
    let index_arg = case.index_arg();
    let args = ["pip", "install", DIST, "--index", index_arg.as_str()];
    let out = case.run(&args);
    case.assert_installed_into_the_project_environment("pip install", &args, &out);
}

/// The same promise through the other entry point: a requirements file synced
/// with no `--python`.
#[test]
fn pip_sync_without_python_installs_into_the_project_environment() {
    let case = Case::with_venv();
    let requirements = case.project.join("requirements.txt");
    std::fs::write(&requirements, "cliapp==1.0\n")
        .unwrap_or_else(|e| panic!("write {}: {e}", requirements.display()));

    let index_arg = case.index_arg();
    let args = [
        "pip",
        "sync",
        "requirements.txt",
        "--index",
        index_arg.as_str(),
    ];
    let out = case.run(&args);
    case.assert_installed_into_the_project_environment("pip sync", &args, &out);
}

/// The derived default must never override a caller who stated an interpreter:
/// `--python <other absolute path>` decides both the shebang and the directory
/// the wrapper is written to, even when a `.venv` is right there.
#[test]
fn the_explicit_python_flag_still_wins_over_the_derived_default() {
    let case = Case::with_venv();
    let alt = case.alternate_interpreter();
    let alt_arg = path_arg(&alt);
    let index_arg = case.index_arg();
    let args = [
        "pip",
        "install",
        DIST,
        "--index",
        index_arg.as_str(),
        "--python",
        alt_arg.as_str(),
    ];
    let out = case.run(&args);
    expect_ok("pip install --python <alt>", &args, &out);
    assert!(
        stdout_of(&out).lines().any(|l| l.trim() == INSTALLED_LINE),
        "the command must report `{INSTALLED_LINE}` on stdout: {}",
        render(&args, &out)
    );

    // `--site-packages` was not given, so the distribution still goes to the
    // project's environment: the flag under test moves the interpreter only.
    let purelib = case.purelib();
    assert!(
        purelib.join(DIST_INFO).is_dir(),
        "the install must still place {DIST_INFO} in {}: it is missing. present there: {:?}\n{}",
        purelib.display(),
        listing(&purelib),
        render(&args, &out)
    );

    let alt_dir = alt
        .parent()
        .expect("the alternate interpreter has a parent directory")
        .to_path_buf();
    let wrapper = alt_dir.join(SCRIPT);
    assert!(
        wrapper.is_file(),
        "with `--python {alt_arg}` the console script must be written beside that interpreter, \
         at {}: it is missing. present there: {:?}\n{}",
        wrapper.display(),
        listing(&alt_dir),
        render(&args, &out)
    );
    let shebang = case.shebang_of(&wrapper, "explicit --python");
    assert_eq!(
        named_abs(Path::new(&shebang)),
        named_abs(&alt),
        "with `--python {alt_arg}` the console script's shebang must name that interpreter, not \
         the environment's own {}; it names {shebang}",
        case.venv_python().display(),
    );
    assert!(
        !case.venv_bin().join(SCRIPT).is_file(),
        "with `--python {alt_arg}` no console script may be written beside the environment's \
         interpreter: {} exists",
        case.venv_bin().join(SCRIPT).display(),
    );
}

/// The fail-closed edge: with no environment to derive an interpreter from and
/// none named, the install refuses, says why, writes no distribution, and does
/// not panic. Deriving from a `.venv` must not turn this into a silent install
/// against some arbitrary interpreter.
#[test]
fn without_an_environment_a_console_script_install_still_refuses_and_installs_nothing() {
    let case = Case::without_venv();
    let site = case.project.join("site-packages");
    std::fs::create_dir_all(&site).unwrap_or_else(|e| panic!("create {}: {e}", site.display()));

    let index_arg = case.index_arg();
    let site_arg = path_arg(&site);
    let args = [
        "pip",
        "install",
        DIST,
        "--index",
        index_arg.as_str(),
        "--site-packages",
        site_arg.as_str(),
    ];
    let out = case.run(&args);

    assert!(
        !out.status.success(),
        "with no `.venv` and no `--python`, installing a wheel that declares a console script \
         must fail rather than guess an interpreter: {}",
        render(&args, &out)
    );
    let stderr = stderr_of(&out);
    assert!(
        !stderr.contains("panicked at") && !stdout_of(&out).contains("panicked at"),
        "the refusal must be a reported error, not a panic: {}",
        render(&args, &out)
    );
    assert!(
        stderr.contains(ABSOLUTE_PATH_REFUSAL),
        "the refusal must name its reason (`{ABSOLUTE_PATH_REFUSAL}`): {}",
        render(&args, &out)
    );

    assert!(
        !site.join(DIST_INFO).exists(),
        "the refused install must leave no {DIST_INFO} in {}: present there: {:?}",
        site.display(),
        listing(&site)
    );
    assert!(
        !site.join(MODULE).exists(),
        "the refused install must leave no `{MODULE}` package in {}: present there: {:?}",
        site.display(),
        listing(&site)
    );
    let mut wrappers = Vec::new();
    find_named(&case.project, SCRIPT, &mut wrappers);
    assert!(
        wrappers.is_empty(),
        "the refused install must write no console script anywhere under {}: found {:?}",
        case.project.display(),
        wrappers
    );

    // The user-visible consequence: nothing is importable from that directory.
    let importer = resolve_python3();
    let probe = Command::new(&importer)
        .args(["-c", &format!("import {MODULE}")])
        .current_dir(&case.project)
        .env("HOME", &case.home)
        .env("PYTHONPATH", &site)
        .env_remove("VIRTUAL_ENV")
        .output()
        .unwrap_or_else(|e| panic!("spawn {}: {e}", importer.display()));
    assert!(
        !probe.status.success(),
        "the refused install must leave `{MODULE}` unimportable from {}; `{} -c 'import {MODULE}'` \
         exited {:?} with stdout {:?}",
        site.display(),
        importer.display(),
        probe.status.code(),
        stdout_of(&probe),
    );
}
