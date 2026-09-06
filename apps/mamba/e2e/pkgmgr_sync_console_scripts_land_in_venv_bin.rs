//! Black-box contract: a console script `mamba sync` installs must land beside
//! the interpreter its own shebang names — the environment's script directory —
//! so the project can run it, and must land nowhere else.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary over a throwaway directory tree and
//! reads only what the binary wrote or what the project's own interpreter
//! reports. The wheel is built by the product's own `WheelBuilder` with an
//! `entry_points.txt` carrying one `[console_scripts]` entry, frozen by `mamba
//! index build`, locked by `mamba add`/`mamba lock`, and installed by `mamba
//! sync`, so a green run means those steps agree on one artifact and one
//! wrapper. Nothing installed is hand-written.
//!
//! # The observation point
//!
//! The script directory is asked of the `.venv` interpreter itself
//! (`sysconfig.get_paths()["scripts"]`), never derived by the case from `.venv`
//! plus a literal `bin`. A product that invents its own directory therefore
//! cannot satisfy the assertions by agreeing with the case's arithmetic, and a
//! layout change in CPython moves the case and the product together.
//!
//! The wrapper is executed as a child process by its own absolute path, with
//! `PATH` removed from the child's environment. There is no shell, no `PATH`
//! lookup, and no `mamba run` in between: what exits 0 and prints the sentinel
//! is the wrapper file at the reported location, or nothing is.
//!
//! # Why the current tree cannot pass
//!
//! `Installer::install` derives the wrapper directory as
//! `site_packages.parent()/bin`. For a PEP 405 environment `site_packages` is
//! `<venv>/lib/pythonX.Y/site-packages`, so every wrapper is written to
//! `<venv>/lib/pythonX.Y/bin/<name>` — a directory no interpreter, launcher, or
//! `PATH` entry ever looks in — while `<venv>/bin` holds only `activate*` and
//! `python*`. Two assertions separately refuse that:
//!
//! 1. the wrapper must exist, executable, at the directory the interpreter
//!    reports as `scripts`; and
//! 2. no file with the script's name may exist anywhere under `<venv>/lib`,
//!    so writing the wrapper to both places, or leaving a copy behind, is
//!    refused as loudly as writing it to the wrong place alone.
//!
//! # Facets
//!
//! - **Behavior**: for both lock origins this project supports offline — a
//!   frozen-index entry and a `direct_file` entry — the wrapper lands in the
//!   reported script directory, is executable, prints the entry point's own
//!   output, and exits 0. The second `mamba sync` is the documented no-op, and
//!   the wrapper survives it: convergence must not delete what it installed.
//! - **Security (the wrapper runs the project's environment, not the host's)**:
//!   a console script is an executable file this tool writes and the user then
//!   runs. The case pins its containment — the shebang names an interpreter
//!   inside the project's `.venv`, the running wrapper reports a `sys.prefix`
//!   equal to that `.venv`, and every file the install wrote with the script's
//!   name is inside the `.venv`. A wrapper whose shebang reached a host
//!   interpreter would import that interpreter's packages, not the ones the
//!   lock pins, so "runs at all" is not enough: it must run *this* environment.
//! - **Performance**: the work item names no budget, so this case asserts no
//!   timing.
//!
//! # Hermeticity
//!
//! No network, ever: the wheel is built in-process and the index is a local
//! directory. `HOME` and `MAMBA_CACHE_DIR` are pinned into the temp tree;
//! `MAMBA_FROZEN_INDEX`, `MAMBA_INDEX_URL`, `MAMBA_JOBS`, `XDG_CACHE_HOME`,
//! `VIRTUAL_ENV`, and `PYTHONPATH` are removed so no ambient value can supply
//! an answer. Nothing is skipped and nothing is `#[ignore]`d: a missing
//! `python3` fails the case naming it.

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

/// The console script's name. Deliberately unlike `DIST` and `MODULE` so the
/// "nothing under `lib/`" sweep cannot collide with the installed package
/// directory or its dist-info and report a false find.
const SCRIPT: &str = "sentinelctl";

/// Printed by the entry point and by nothing else, so reading it back off a
/// child process's stdout proves the wheel's own function ran.
const SENTINEL: &str = "mamba-e2e-4229-console-script-ran";

/// The module the entry point names. `main` prints the sentinel, then the two
/// facts that identify which environment executed it, then returns 0 — the
/// wrapper's `sys.exit(main())` turns that into exit status 0.
const CLI_BODY: &str = "import sys\n\
                        \n\
                        SENTINEL = \"mamba-e2e-4229-console-script-ran\"\n\
                        \n\
                        \n\
                        def main():\n\
                        \x20   print(SENTINEL)\n\
                        \x20   print(sys.executable)\n\
                        \x20   print(sys.prefix)\n\
                        \x20   return 0\n";

/// One `[console_scripts]` entry, in the `entry_points.txt` shape PEP 517
/// backends emit and `flask = flask.cli:main` is the canonical example of.
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

fn path_arg(path: &Path) -> String {
    path.to_str()
        .unwrap_or_else(|| panic!("fixture: path {} is not utf-8", path.display()))
        .to_string()
}

/// Build the real, importable wheel through the product's own wheel builder,
/// including the `entry_points.txt` body that declares the console script.
fn build_wheel(out_dir: &Path) -> PathBuf {
    let filename = compose_filename(DIST, VERSION, "py3", "none", "any");
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-console-scripts");
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

/// One scaffolded project: `mamba init`, a real PEP 405 environment from the
/// host `python3`, and the script directory that environment reports.
struct Case {
    _root: tempfile::TempDir,
    root: PathBuf,
    project: PathBuf,
    home: PathBuf,
    scripts: PathBuf,
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
            scripts: PathBuf::new(),
        };
        let scripts = case.reported_scripts_dir();
        Case { scripts, ..case }
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

    /// Ask the environment's own interpreter where a console script belongs.
    /// This is the case's only definition of "the script directory": it is
    /// never `.venv` plus a literal `bin`.
    fn reported_scripts_dir(&self) -> PathBuf {
        let python = self.venv_python();
        let out = Command::new(&python)
            .args([
                "-c",
                "import sysconfig; print(sysconfig.get_paths()['scripts'])",
            ])
            .current_dir(&self.project)
            .env("HOME", &self.home)
            .env_remove("PYTHONPATH")
            .env_remove("VIRTUAL_ENV")
            .output()
            .unwrap_or_else(|e| panic!("spawn {}: {e}", python.display()));
        assert!(
            out.status.success(),
            "fixture: `{} -c 'print scripts'` failed: {}",
            python.display(),
            String::from_utf8_lossy(&out.stderr)
        );
        let scripts = PathBuf::from(stdout_of(&out).trim());
        assert!(
            scripts.is_dir(),
            "fixture: the environment reported the script directory {}, which is not a directory",
            scripts.display()
        );
        assert!(
            canon(&scripts).starts_with(canon(&self.venv())),
            "fixture: the environment reported the script directory {}, which is outside {}",
            scripts.display(),
            self.venv().display()
        );
        scripts
    }

    fn run(&self, args: &[&str]) -> Output {
        run(&self.project, &self.home, args)
    }

    /// Import the entry point's own module through the environment's
    /// interpreter and call it. This is the fixture self-check: it is green on
    /// the tree as it stands, because `mamba sync` already installs the
    /// distribution correctly. Its job is to make a failure below attributable
    /// to where the wrapper was written and to nothing else — not to a wheel
    /// that never installed, a module that does not import, or an entry point
    /// that does not run.
    fn assert_entry_point_is_importable(&self, origin: &str) {
        let python = self.venv_python();
        let probe = format!("import sys; from {MODULE}.cli import main; sys.exit(main())");
        let out = Command::new(&python)
            .args(["-c", probe.as_str()])
            .current_dir(&self.project)
            .env("HOME", &self.home)
            .env_remove("PYTHONPATH")
            .env_remove("VIRTUAL_ENV")
            .output()
            .unwrap_or_else(|e| panic!("spawn {}: {e}", python.display()));
        assert!(
            out.status.success(),
            "[{origin}] fixture: the installed entry point must import and run through {}: \
             exited {:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
            python.display(),
            out.status.code(),
            stdout_of(&out),
            String::from_utf8_lossy(&out.stderr),
        );
        assert_eq!(
            stdout_of(&out).lines().next(),
            Some(SENTINEL),
            "[{origin}] fixture: calling `{MODULE}.cli:main` must print the sentinel; got {:?}",
            stdout_of(&out)
        );
    }

    /// Execute the wrapper file itself: absolute path, no shell, and no `PATH`
    /// in the child's environment, so nothing but the file at `path` can
    /// answer. Returns its stdout lines.
    fn run_wrapper(&self, path: &Path, origin: &str) -> Vec<String> {
        let out = Command::new(path)
            .current_dir(&self.project)
            .env("HOME", &self.home)
            .env_remove("PATH")
            .env_remove("PYTHONPATH")
            .env_remove("PYTHONHOME")
            .env_remove("VIRTUAL_ENV")
            .output()
            .unwrap_or_else(|e| panic!("[{origin}] spawn the wrapper {}: {e}", path.display()));
        assert!(
            out.status.success(),
            "[{origin}] running the console script {} must exit 0; exited {:?}\
             \n--- stdout ---\n{}\n--- stderr ---\n{}\n--- the wrapper's own bytes ---\n{}",
            path.display(),
            out.status.code(),
            stdout_of(&out),
            stderr_of(&out),
            std::fs::read_to_string(path).unwrap_or_else(|e| format!("<unreadable: {e}>")),
        );
        stdout_of(&out).lines().map(|l| l.to_string()).collect()
    }

    /// Everything the work item promises about one installed console script.
    /// `origin` names the lock shape so a failure says which one broke.
    fn assert_console_script_landed_beside_the_interpreter(&self, origin: &str) {
        self.assert_entry_point_is_importable(origin);

        let wrapper = self.scripts.join(SCRIPT);
        let mut elsewhere = Vec::new();
        find_named(&self.venv(), SCRIPT, &mut elsewhere);
        // The interpreter answers with the path it resolved (on macOS
        // `/var/folders/...` reports as `/private/var/folders/...`), so the
        // two spellings only compare after canonicalization.
        let wrapper_canon = canon(&wrapper);
        elsewhere.retain(|p| canon(p) != wrapper_canon);

        assert!(
            wrapper.is_file(),
            "[{origin}] `mamba sync` must write the console script `{SCRIPT}` to the script \
             directory the environment itself reports, {}, beside the interpreter its shebang \
             names: {} is missing.\n  present in that directory: {:?}\n  files named `{SCRIPT}` \
             found elsewhere under {}: {:?}",
            self.scripts.display(),
            wrapper.display(),
            listing(&self.scripts),
            self.venv().display(),
            elsewhere,
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

        // Containment: the shebang must name an interpreter inside this
        // project's environment, never a host one whose packages the lock does
        // not pin.
        let body = std::fs::read_to_string(&wrapper)
            .unwrap_or_else(|e| panic!("[{origin}] read {}: {e}", wrapper.display()));
        let shebang = body
            .lines()
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
            .to_string();
        // `.venv/bin/python` is a symlink to the base interpreter, so the
        // directory holding it is what identifies the environment;
        // canonicalizing the executable itself resolves out of the venv on
        // every host that seeds one from a version manager.
        let shebang_path = PathBuf::from(&shebang);
        let shebang_dir = canon(shebang_path.parent().unwrap_or_else(|| Path::new("")));
        assert!(
            shebang_dir.starts_with(canon(&self.venv())),
            "[{origin}] the console script's shebang must name an interpreter inside {}, got {}",
            self.venv().display(),
            shebang,
        );

        // The behavior: the file at that path runs, says what the entry point
        // says, and exits 0.
        let lines = self.run_wrapper(&wrapper, origin);
        assert!(
            lines.len() >= 3,
            "[{origin}] the console script must print the sentinel, its interpreter, and its \
             prefix; got {lines:?}"
        );
        assert_eq!(
            lines[0], SENTINEL,
            "[{origin}] the console script must run the wheel's own entry point, which prints \
             the sentinel; got {:?}",
            lines[0]
        );
        assert_eq!(
            canon(Path::new(&lines[2])),
            canon(&self.venv()),
            "[{origin}] the console script must run inside the project's environment: it \
             reported sys.prefix {} and sys.executable {}",
            lines[2],
            lines[1],
        );

        // Nothing under `lib/`: not instead of the script directory, and not
        // as well as it.
        let mut under_lib = Vec::new();
        find_named(&self.venv().join("lib"), SCRIPT, &mut under_lib);
        assert!(
            under_lib.is_empty(),
            "[{origin}] `mamba sync` must write no console script under the environment's \
             library tree {}: found {:?}",
            self.venv().join("lib").display(),
            under_lib,
        );

        // Convergence must not delete what it installed.
        let sync_args = ["sync"];
        let out = self.run(&sync_args);
        expect_ok(&format!("[{origin}] second sync"), &sync_args, &out);
        assert!(
            wrapper.is_file(),
            "[{origin}] the second `mamba sync` must leave the console script in place: {} is \
             gone. {}",
            wrapper.display(),
            render(&sync_args, &out)
        );
        let lines = self.run_wrapper(&wrapper, &format!("{origin}, after the second sync"));
        assert_eq!(
            lines.first().map(String::as_str),
            Some(SENTINEL),
            "[{origin}] the console script must still run after the second `mamba sync`; got \
             {lines:?}"
        );
    }
}

#[test]
fn frozen_index_entry_installs_the_console_script_into_the_reported_script_directory() {
    let case = Case::start();
    let wheels = case.root.join("wheels");
    let index = case.root.join("index");
    let wheel = build_wheel(&wheels);

    let index_arg = path_arg(&index);
    let wheel_arg = path_arg(&wheel);
    let build_args = [
        "index",
        "build",
        "--out",
        index_arg.as_str(),
        wheel_arg.as_str(),
    ];
    let out = case.run(&build_args);
    expect_ok("index build", &build_args, &out);

    let add_args = ["add", DIST, "--index", index_arg.as_str()];
    let out = case.run(&add_args);
    expect_ok("add --index", &add_args, &out);

    let lock_args = ["lock", "--index", index_arg.as_str()];
    let out = case.run(&lock_args);
    expect_ok("lock --index", &lock_args, &out);

    // Fixture self-check: the entry this sub-case is about must be the one the
    // lock carries, so a resolver change fails here rather than as a wrapper
    // assertion below.
    let lock = std::fs::read_to_string(case.project.join("mamba.lock")).expect("read mamba.lock");
    assert!(
        lock.contains("source_kind = \"index\"") && lock.contains("path = "),
        "fixture: `mamba add --index` then `mamba lock --index` must leave an index entry with \
         a path\n--- mamba.lock ---\n{lock}"
    );

    let sync_args = ["sync"];
    let out = case.run(&sync_args);
    expect_ok("sync", &sync_args, &out);

    case.assert_console_script_landed_beside_the_interpreter("index");
}

#[test]
fn direct_file_entry_installs_the_console_script_into_the_reported_script_directory() {
    let case = Case::start();
    let wheel = build_wheel(&case.project.join("wheels"));
    assert!(
        wheel.is_file(),
        "fixture: {} is not a file",
        wheel.display()
    );

    let add_args = ["add", "./wheels/cliapp-1.0-py3-none-any.whl"];
    let out = case.run(&add_args);
    expect_ok("add ./wheels/<wheel>", &add_args, &out);

    let lock = std::fs::read_to_string(case.project.join("mamba.lock")).expect("read mamba.lock");
    assert!(
        lock.contains("source_kind = \"direct_file\""),
        "fixture: `mamba add <wheel>` must lock a direct_file entry\n--- mamba.lock ---\n{lock}"
    );

    let sync_args = ["sync"];
    let out = case.run(&sync_args);
    expect_ok("sync", &sync_args, &out);

    case.assert_console_script_landed_beside_the_interpreter("direct_file");
}
