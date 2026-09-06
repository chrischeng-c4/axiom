//! Black-box contract: what `mamba sync` installs, `mamba remove` plus the
//! next `mamba sync` must take away — including the console-script wrapper.
//! An installer that writes an executable it does not record cannot uninstall
//! it, and the environment keeps a runnable file no lock pins.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary over a throwaway directory tree
//! and reads only what the binary wrote or what the project's own interpreter
//! reports. Each wheel is built in-process by the product's own `WheelBuilder`
//! with an `entry_points.txt` carrying one `[console_scripts]` entry, frozen by
//! `mamba index build`, pinned by `mamba add`/`mamba lock`, installed by
//! `mamba sync`, dropped by `mamba remove`, and converged by the following
//! `mamba sync`. Nothing installed, recorded, or removed is hand-written, and
//! the case links neither the resolver, the installer, the uninstaller, nor
//! the sync planner: `mamba::pkgmanage::pkgmgr::wheel_build` builds the
//! fixture's input and nothing else.
//!
//! # The observation point
//!
//! Two directories are asked of the `.venv` interpreter itself, never derived
//! by the case from `.venv` plus a literal name:
//!
//! | what | asked as |
//! |---|---|
//! | the script directory | `sysconfig.get_paths()["scripts"]` |
//! | the site-packages directory | `sysconfig.get_paths()["purelib"]` |
//!
//! So a product that invents its own layout cannot satisfy these assertions by
//! agreeing with the case's arithmetic, and a CPython layout change moves the
//! case and the product together.
//!
//! A wrapper is executed as a child process by its own absolute path with
//! `PATH` removed from the child's environment. There is no shell, no `PATH`
//! lookup and no `mamba run` in between: what exits 0 and prints the sentinel
//! is the file at the reported location, or nothing is. Absence is read with
//! `symlink_metadata`, so a dangling symlink left in the script directory
//! counts as present rather than as removed.
//!
//! # Why the current tree cannot pass
//!
//! `Installer::install` writes one wrapper per console script into the
//! interpreter's own directory, then renders the installed `RECORD` from the
//! rows it parsed out of the *wheel's* `RECORD`. The wrapper is in neither, so
//! the installed `RECORD` never names it. `prune_distribution` removes exactly
//! what that `RECORD` lists and then sweeps only under site-packages, so the
//! wrapper is unreachable from every removal path the product has. Measured on
//! the tree this case was written against: after `mamba remove cliapp` and
//! `mamba sync`, both exit 0, `cliapp-1.0.dist-info` is gone, and
//! `.venv/bin/sentinelctl` is still there — running it exits 1 with
//! `ModuleNotFoundError: No module named 'cliapp'` on its last stderr line,
//! while `mamba sync --check` still reports `environment is synchronized with
//! mamba.lock` and exits 0.
//!
//! # Facets
//!
//! - **Behavior**: after `remove` + `sync` the wrapper path is gone, no file
//!   carrying its name survives anywhere under the environment, and the
//!   distribution's dist-info is gone with it. The mechanism is observed
//!   directly one step earlier: the installed `RECORD` carries exactly one row
//!   that resolves onto the wrapper, relative to site-packages, with the
//!   sha256 and byte length of the bytes on disk — PEP 376's contract, and
//!   pip's, that an installer records what it wrote. Re-entry is observed
//!   after the removal: the environment still converges, `sync --check` still
//!   agrees, and re-adding the distribution brings a working wrapper back.
//! - **Security (blast radius, in both directions)**: an orphaned wrapper is a
//!   runnable file inside an activated environment that no lock pins and no
//!   dist-info explains — a name a later, different distribution can claim,
//!   and today a stale executable whose only sign of trouble is a traceback.
//!   Removing it must nevertheless stay bounded to what this distribution
//!   owns: a sibling distribution's wrapper in the same directory must survive
//!   intact and still run, so a name-guessing sweep or a wholesale `bin/`
//!   wipe — which would also take `python` and `activate` — fails here as
//!   loudly as leaving the orphan behind. The wrapper row is required to be
//!   *relative* to site-packages, so a `RECORD` cannot be made to name an
//!   arbitrary absolute path that a later uninstall would delete.
//! - **Performance**: this work item names no budget, so the case asserts no
//!   timing, no process count, and no concurrency.
//!
//! # Hermeticity
//!
//! No network, ever: every wheel is built in-process and the only index is a
//! local directory this run wrote. `HOME` and `MAMBA_CACHE_DIR` are pinned
//! into each test's own temp tree, so no artifact or metadata cached by
//! another run — or by another case — can stand in for an install;
//! `MAMBA_FROZEN_INDEX`, `MAMBA_INDEX_URL`, `MAMBA_JOBS`, `XDG_CACHE_HOME`,
//! `VIRTUAL_ENV`, and `PYTHONPATH` are removed, so no ambient value can supply
//! an answer. Nothing is skipped and nothing is `#[ignore]`d: a host without
//! `python3` fails this case naming it.

use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};

use base64::Engine;
use mamba::pkgmanage::pkgmgr::wheel_build::{
    compose_filename, CoreMetadata, WheelBuilder, WheelMetadata,
};
use sha2::{Digest, Sha256};

/// One version for every distribution the case builds, so a dist-info
/// directory name is `<dist>-1.0.dist-info` throughout.
const VERSION: &str = "1.0";

/// A distribution the case builds, installs, and (for one of them) removes.
/// `script` is deliberately unlike `dist` and `module` so a sweep for the
/// wrapper's name cannot collide with the installed package directory or its
/// dist-info and report a false find.
struct Package {
    dist: &'static str,
    module: &'static str,
    script: &'static str,
    sentinel: &'static str,
}

/// The distribution under test: the one the user removes.
const CLIAPP: Package = Package {
    dist: "cliapp",
    module: "cliapp",
    script: "sentinelctl",
    sentinel: "sentinel ok",
};

/// The survivor: a second distribution whose wrapper shares the script
/// directory and must come through the removal untouched.
const OTHERAPP: Package = Package {
    dist: "otherapp",
    module: "otherapp",
    script: "otherctl",
    sentinel: "other ok",
};

/// The module the entry point names. `main` prints the sentinel, then the two
/// facts that identify which environment executed it, then returns 0 — the
/// wrapper's `sys.exit(main())` turns that into exit status 0.
fn cli_body(pkg: &Package) -> String {
    format!(
        "import sys\n\
         \n\
         SENTINEL = \"{}\"\n\
         \n\
         \n\
         def main():\n\
         \x20   print(SENTINEL)\n\
         \x20   print(sys.executable)\n\
         \x20   print(sys.prefix)\n\
         \x20   return 0\n",
        pkg.sentinel
    )
}

/// One `[console_scripts]` entry, in the `entry_points.txt` shape PEP 517
/// backends emit and `flask = flask.cli:main` is the canonical example of.
fn entry_points_txt(pkg: &Package) -> String {
    format!(
        "[console_scripts]\n{} = {}.cli:main\n",
        pkg.script, pkg.module
    )
}

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

/// Resolve `.` and `..` without touching the filesystem. Only ever used as the
/// fallback below, for a path that does not exist — and a path that does not
/// exist is not the wrapper, which does.
fn lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// The one spelling every path comparison in this case uses. The interpreter
/// answers with the path it resolved (on macOS `/var/folders/...` reports as
/// `/private/var/folders/...`), and a `RECORD` row climbs out of site-packages
/// with `..` segments, so two spellings of one file only compare after this.
fn canon(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| lexical(path))
}

/// A path exists in *any* form, including as a dangling symlink: `Path::exists`
/// follows links and would read a broken leftover as removed.
fn exists_any(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok()
}

fn path_arg(path: &Path) -> String {
    path.to_str()
        .unwrap_or_else(|| panic!("fixture: path {} is not utf-8", path.display()))
        .to_string()
}

/// The urlsafe-base64, unpadded SHA-256 of `bytes` — the `RECORD` hash field's
/// encoding, as PEP 376 and pip's `RECORD` writer define it.
fn sha256_b64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(bytes))
}

/// One `RECORD` row, split but not interpreted: `path,hash,size`.
struct Row {
    path: String,
    hash: String,
    size: String,
}

/// Split a `RECORD` body into rows. The case does its own splitting rather
/// than calling the product's parser, so a parser that learned to invent or
/// drop a row cannot answer for the file on disk.
fn parse_record(text: &str) -> Vec<Row> {
    text.lines()
        .map(|raw| raw.trim_end_matches('\r'))
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let mut fields = line.splitn(3, ',');
            Row {
                path: fields.next().unwrap_or_default().to_string(),
                hash: fields.next().unwrap_or_default().to_string(),
                size: fields.next().unwrap_or_default().to_string(),
            }
        })
        .collect()
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

/// Build a real, importable wheel through the product's own wheel builder,
/// including the `entry_points.txt` body that declares the console script.
fn build_wheel(out_dir: &Path, pkg: &Package) -> PathBuf {
    let filename = compose_filename(pkg.dist, VERSION, "py3", "none", "any");
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-remove-console-scripts");
    wheel_meta.tags.push("py3-none-any".into());
    let core_meta = CoreMetadata::new(pkg.dist, VERSION);
    let mut builder = WheelBuilder::new(filename, wheel_meta, core_meta);
    builder.add_file(format!("{}/__init__.py", pkg.module), String::new());
    builder.add_file(format!("{}/cli.py", pkg.module), cli_body(pkg));
    builder.set_entry_points(entry_points_txt(pkg));
    let wheel = builder
        .build_to_dir(out_dir)
        .unwrap_or_else(|e| panic!("fixture: build wheel {}-{VERSION}: {e:?}", pkg.dist));
    let expected = format!("{}-{VERSION}-py3-none-any.whl", pkg.dist);
    assert_eq!(
        wheel.file_name().and_then(|n| n.to_str()),
        Some(expected.as_str()),
        "fixture: the wheel builder named the artifact {}",
        wheel.display()
    );
    wheel
}

/// One scaffolded project: `mamba init`, a real PEP 405 environment from the
/// host `python3`, a frozen local index holding the fixture's wheels, and the
/// two directories that environment reports for scripts and for packages.
struct Case {
    _root: tempfile::TempDir,
    project: PathBuf,
    home: PathBuf,
    index: PathBuf,
    scripts: PathBuf,
    site: PathBuf,
}

impl Case {
    fn start(packages: &[&Package]) -> Case {
        let root_dir = tempfile::tempdir().expect("create temp root");
        let root = root_dir.path().to_path_buf();
        let project = root.join("project");
        let home = root.join("home");
        let wheels = root.join("wheels");
        let index = root.join("index");
        for dir in [&project, &home, &wheels] {
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

        let built: Vec<String> = packages
            .iter()
            .map(|pkg| path_arg(&build_wheel(&wheels, pkg)))
            .collect();
        let index_arg = path_arg(&index);
        let mut build_args = vec!["index", "build", "--out", index_arg.as_str()];
        for wheel in &built {
            build_args.push(wheel.as_str());
        }
        let out = run(&project, &home, &build_args);
        expect_ok("index build", &build_args, &out);

        let case = Case {
            _root: root_dir,
            project,
            home,
            index,
            scripts: PathBuf::new(),
            site: PathBuf::new(),
        };
        let scripts = case.reported_path("scripts");
        let site = case.reported_path("purelib");
        Case {
            scripts,
            site,
            ..case
        }
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

    /// Ask the environment's own interpreter for one of its install paths.
    /// This is the case's only definition of "the script directory" and of
    /// "the site-packages directory": neither is ever `.venv` plus a literal
    /// name.
    fn reported_path(&self, key: &str) -> PathBuf {
        let python = self.venv_python();
        let code = format!("import sysconfig; print(sysconfig.get_paths()['{key}'])");
        let out = Command::new(&python)
            .args(["-c", code.as_str()])
            .current_dir(&self.project)
            .env("HOME", &self.home)
            .env_remove("PYTHONPATH")
            .env_remove("VIRTUAL_ENV")
            .output()
            .unwrap_or_else(|e| panic!("spawn {}: {e}", python.display()));
        assert!(
            out.status.success(),
            "fixture: `{} -c 'print {key}'` failed: {}",
            python.display(),
            String::from_utf8_lossy(&out.stderr)
        );
        let dir = PathBuf::from(stdout_of(&out).trim());
        assert!(
            dir.is_dir(),
            "fixture: the environment reported the `{key}` directory {}, which is not a directory",
            dir.display()
        );
        assert!(
            canon(&dir).starts_with(canon(&self.venv())),
            "fixture: the environment reported the `{key}` directory {}, which is outside {}",
            dir.display(),
            self.venv().display()
        );
        dir
    }

    fn run(&self, args: &[&str]) -> Output {
        run(&self.project, &self.home, args)
    }

    fn index_arg(&self) -> String {
        path_arg(&self.index)
    }

    /// `add` each distribution against the frozen index, `lock`, then `sync`.
    /// Every step must exit 0: an install that did not happen is a fixture
    /// failure, not a contract failure.
    fn install(&self, packages: &[&Package], origin: &str) -> Output {
        let index_arg = self.index_arg();
        for pkg in packages {
            let add_args = ["add", pkg.dist, "--index", index_arg.as_str()];
            let out = self.run(&add_args);
            expect_ok(
                &format!("[{origin}] add {} --index", pkg.dist),
                &add_args,
                &out,
            );
        }
        let lock_args = ["lock", "--index", index_arg.as_str()];
        let out = self.run(&lock_args);
        expect_ok(&format!("[{origin}] lock --index"), &lock_args, &out);
        self.sync(origin)
    }

    fn sync(&self, origin: &str) -> Output {
        let sync_args = ["sync"];
        let out = self.run(&sync_args);
        expect_ok(&format!("[{origin}] sync"), &sync_args, &out);
        out
    }

    fn remove(&self, pkg: &Package, origin: &str) -> Output {
        let remove_args = ["remove", pkg.dist];
        let out = self.run(&remove_args);
        expect_ok(
            &format!("[{origin}] remove {}", pkg.dist),
            &remove_args,
            &out,
        );
        out
    }

    fn wrapper(&self, pkg: &Package) -> PathBuf {
        self.scripts.join(pkg.script)
    }

    fn dist_info(&self, pkg: &Package) -> PathBuf {
        self.site.join(format!("{}-{VERSION}.dist-info", pkg.dist))
    }

    /// Execute the wrapper file itself: absolute path, no shell, and no `PATH`
    /// in the child's environment, so nothing but the file at `path` can
    /// answer.
    fn exec_wrapper(&self, path: &Path) -> Output {
        Command::new(path)
            .current_dir(&self.project)
            .env("HOME", &self.home)
            .env_remove("PATH")
            .env_remove("PYTHONPATH")
            .env_remove("PYTHONHOME")
            .env_remove("VIRTUAL_ENV")
            .output()
            .unwrap_or_else(|e| panic!("spawn the wrapper {}: {e}", path.display()))
    }

    /// The wrapper must run, in this environment, and print its own sentinel.
    fn assert_wrapper_runs(&self, pkg: &Package, origin: &str) {
        let path = self.wrapper(pkg);
        let out = self.exec_wrapper(&path);
        assert!(
            out.status.success(),
            "[{origin}] running the console script {} must exit 0; exited {:?}\
             \n--- stdout ---\n{}\n--- stderr ---\n{}\n--- the wrapper's own bytes ---\n{}",
            path.display(),
            out.status.code(),
            stdout_of(&out),
            stderr_of(&out),
            std::fs::read_to_string(&path).unwrap_or_else(|e| format!("<unreadable: {e}>")),
        );
        let lines: Vec<String> = stdout_of(&out).lines().map(|l| l.to_string()).collect();
        assert_eq!(
            lines.first().map(String::as_str),
            Some(pkg.sentinel),
            "[{origin}] the console script {} must run `{}.cli:main`, which prints `{}`; got \
             {lines:?}",
            path.display(),
            pkg.module,
            pkg.sentinel,
        );
        assert_eq!(
            lines.get(2).map(|p| canon(Path::new(p))),
            Some(canon(&self.venv())),
            "[{origin}] the console script {} must run inside the project's environment; it \
             reported {lines:?}",
            path.display(),
        );
    }

    /// What a leftover file at `path` does when it is executed, folded into a
    /// failure message so a red says what survived rather than only that
    /// something did.
    fn describe_leftover(&self, path: &Path) -> String {
        if !exists_any(path) {
            return "it is absent".to_string();
        }
        let out = self.exec_wrapper(path);
        format!(
            "it still exists; running it exits {:?} with the last stderr line {:?}",
            out.status.code(),
            stderr_of(&out).lines().last().unwrap_or("").to_string(),
        )
    }

    /// The fixture self-check every test starts from: the distribution is
    /// installed, its wrapper is at the reported script directory, and it
    /// runs. This is green on the tree as it stands, so a red below is
    /// attributable to the removal and to nothing else.
    fn assert_installed(&self, pkg: &Package, origin: &str) {
        let dist_info = self.dist_info(pkg);
        assert!(
            dist_info.is_dir(),
            "[{origin}] fixture: `mamba sync` must install {}: {} is missing.\n  \
             site-packages {} holds: {:?}",
            pkg.dist,
            dist_info.display(),
            self.site.display(),
            listing(&self.site),
        );
        let wrapper = self.wrapper(pkg);
        assert!(
            wrapper.is_file(),
            "[{origin}] fixture: `mamba sync` must write the console script `{}` to the script \
             directory the environment itself reports, {}: {} is missing.\n  that directory \
             holds: {:?}",
            pkg.script,
            self.scripts.display(),
            wrapper.display(),
            listing(&self.scripts),
        );
        self.assert_wrapper_runs(pkg, origin);
    }
}

/// The Goal: `remove` then `sync` must take the wrapper away with the
/// distribution.
#[test]
fn remove_then_sync_leaves_no_console_script_behind() {
    let case = Case::start(&[&CLIAPP]);
    case.install(&[&CLIAPP], "install");
    case.assert_installed(&CLIAPP, "install");

    let wrapper = case.wrapper(&CLIAPP);
    let dist_info = case.dist_info(&CLIAPP);

    case.remove(&CLIAPP, "remove");
    case.sync("converge");

    assert!(
        !exists_any(&dist_info),
        "`mamba remove {}` then `mamba sync` must leave no dist-info under {}: {} is still \
         there.\n  site-packages holds: {:?}",
        CLIAPP.dist,
        case.site.display(),
        dist_info.display(),
        listing(&case.site),
    );

    assert!(
        !exists_any(&wrapper),
        "`mamba remove {}` then `mamba sync` must remove the console script it installed, {}, \
         but {}.\n  the script directory {} holds: {:?}\n  the removed distribution's dist-info \
         {} is {}",
        CLIAPP.dist,
        wrapper.display(),
        case.describe_leftover(&wrapper),
        case.scripts.display(),
        listing(&case.scripts),
        dist_info.display(),
        if exists_any(&dist_info) {
            "still present"
        } else {
            "gone, so the wrapper outlived the distribution that owned it"
        },
    );

    // Not moved, not renamed, not left as a dangling link: nothing anywhere
    // under the environment still carries the script's name.
    let mut survivors = Vec::new();
    find_named(&case.venv(), CLIAPP.script, &mut survivors);
    assert!(
        survivors.is_empty(),
        "after `mamba remove {}` and `mamba sync`, no file named `{}` may survive anywhere under \
         {}: found {survivors:?}",
        CLIAPP.dist,
        CLIAPP.script,
        case.venv().display(),
    );
}

/// The mechanism, observed one step before the removal: an installer records
/// what it wrote. PEP 376's `RECORD` is that record, and it is what the
/// RECORD-driven uninstall can act on.
#[test]
fn the_installed_record_names_the_console_script_it_wrote() {
    let case = Case::start(&[&CLIAPP]);
    case.install(&[&CLIAPP], "install");
    case.assert_installed(&CLIAPP, "install");

    let wrapper = case.wrapper(&CLIAPP);
    let wrapper_bytes = std::fs::read(&wrapper)
        .unwrap_or_else(|e| panic!("read the console script {}: {e}", wrapper.display()));
    let wrapper_canon = canon(&wrapper);

    let record_path = case.dist_info(&CLIAPP).join("RECORD");
    let record_text = std::fs::read_to_string(&record_path).unwrap_or_else(|e| {
        panic!(
            "`mamba sync` must leave an installed RECORD at {}: {e}",
            record_path.display()
        )
    });
    let rows = parse_record(&record_text);

    let absolute: Vec<&str> = rows
        .iter()
        .map(|row| row.path.as_str())
        .filter(|path| Path::new(path).is_absolute())
        .collect();
    assert!(
        absolute.is_empty(),
        "every row of {} must name a path relative to the site-packages directory {}; these are \
         absolute: {absolute:?}\n--- RECORD ---\n{record_text}",
        record_path.display(),
        case.site.display(),
    );

    let matching: Vec<&Row> = rows
        .iter()
        .filter(|row| canon(&case.site.join(&row.path)) == wrapper_canon)
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "the installed RECORD {} must carry exactly one row resolving, from the site-packages \
         directory {}, onto the console script `mamba sync` wrote at {}; {} do.\n  rows: {:?}\n\
         --- RECORD ---\n{record_text}",
        record_path.display(),
        case.site.display(),
        wrapper.display(),
        matching.len(),
        rows.iter().map(|row| row.path.as_str()).collect::<Vec<_>>(),
    );
    let row = matching[0];

    assert_eq!(
        row.hash,
        format!("sha256={}", sha256_b64url(&wrapper_bytes)),
        "the RECORD row `{}` for the console script {} must carry the sha256 of the bytes on \
         disk, urlsafe-base64 and unpadded\n--- the wrapper's own bytes ---\n{}",
        row.path,
        wrapper.display(),
        String::from_utf8_lossy(&wrapper_bytes),
    );
    assert_eq!(
        row.size,
        wrapper_bytes.len().to_string(),
        "the RECORD row `{}` for the console script {} must carry its byte length",
        row.path,
        wrapper.display(),
    );
}

/// The boundary: the removal is bounded to what the removed distribution owns.
/// A sibling's wrapper shares the directory and must come through untouched —
/// so a name sweep, a shebang scan, or a wholesale `bin/` wipe fails here.
#[test]
fn removing_one_distribution_keeps_a_siblings_console_script() {
    let case = Case::start(&[&CLIAPP, &OTHERAPP]);
    case.install(&[&CLIAPP, &OTHERAPP], "install");
    case.assert_installed(&CLIAPP, "install");
    case.assert_installed(&OTHERAPP, "install");

    let removed = case.wrapper(&CLIAPP);
    let survivor = case.wrapper(&OTHERAPP);
    let survivor_bytes = std::fs::read(&survivor)
        .unwrap_or_else(|e| panic!("read the console script {}: {e}", survivor.display()));

    case.remove(&CLIAPP, "remove");
    case.sync("converge");

    assert!(
        survivor.is_file(),
        "`mamba remove {}` must not touch `{}`, the console script {} owns: {} is missing after \
         the following `mamba sync`.\n  the script directory {} holds: {:?}",
        CLIAPP.dist,
        OTHERAPP.script,
        OTHERAPP.dist,
        survivor.display(),
        case.scripts.display(),
        listing(&case.scripts),
    );
    assert_eq!(
        std::fs::read(&survivor).ok().as_deref(),
        Some(survivor_bytes.as_slice()),
        "the surviving console script {} must be byte-identical across another distribution's \
         removal",
        survivor.display(),
    );
    assert!(
        case.dist_info(&OTHERAPP).is_dir(),
        "`mamba remove {}` must leave {} installed: {} is missing.\n  site-packages holds: {:?}",
        CLIAPP.dist,
        OTHERAPP.dist,
        case.dist_info(&OTHERAPP).display(),
        listing(&case.site),
    );
    case.assert_wrapper_runs(&OTHERAPP, "after removing the sibling");

    assert!(
        !exists_any(&removed),
        "`mamba remove {}` then `mamba sync` must remove the console script `{}`, {}, but {}.\n  \
         the script directory {} holds: {:?}",
        CLIAPP.dist,
        CLIAPP.script,
        removed.display(),
        case.describe_leftover(&removed),
        case.scripts.display(),
        listing(&case.scripts),
    );
}

/// Re-entry: the environment the removal leaves behind is still one this tool
/// agrees with, and re-adding the distribution brings a working console script
/// back. A removal that leaves the environment un-convergent, or a recorded
/// row a later `sync` cannot act on, fails here.
#[test]
fn the_environment_still_converges_and_the_console_script_returns_on_a_re_add() {
    let case = Case::start(&[&CLIAPP]);
    case.install(&[&CLIAPP], "install");
    case.assert_installed(&CLIAPP, "install");

    // The reference: what a `sync` prints over an environment this tool
    // already considers synchronized. Read off the product, never spelled out
    // here, so the comparison below cannot drift from the product's wording.
    let already_synchronized = stdout_of(&case.sync("second sync"));

    case.remove(&CLIAPP, "remove");
    case.sync("converge");

    let after_removal = stdout_of(&case.sync("sync after the removal"));
    assert_eq!(
        after_removal, already_synchronized,
        "after `mamba remove {}` and the converging `mamba sync`, a further `mamba sync` must \
         report the environment already synchronized, exactly as it does over an untouched \
         install",
        CLIAPP.dist,
    );

    let check_args = ["sync", "--check"];
    let out = case.run(&check_args);
    expect_ok("sync --check after the removal", &check_args, &out);

    // And the distribution can come back.
    case.install(&[&CLIAPP], "re-add");
    case.assert_installed(&CLIAPP, "re-add");
}
