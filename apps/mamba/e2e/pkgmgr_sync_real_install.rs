//! Black-box contract: `mamba sync` must install the lock's real wheels into
//! the environment's own site-packages, so the project interpreter imports
//! them the way every other Python tool does — through `site`, with nothing on
//! `PYTHONPATH`.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary over a throwaway directory tree
//! and reads only what the binary wrote or what the project's own interpreter
//! reports. It never links a product type and never hand-writes an installed
//! file: the wheel is built by the product's own `WheelBuilder`, frozen by
//! `mamba index build`, locked by `mamba add`, and installed by `mamba sync`,
//! so a green run means those four steps agree on one artifact. The one
//! hand-written artifact is a `mamba.lock` for the registry (`url` plus
//! `sha256`) and the malformed-entry sub-cases: the lock is a documented file,
//! and registry resolution is not what this case judges.
//!
//! # The observation point
//!
//! `purelib` is asked of the `.venv` interpreter itself
//! (`sysconfig.get_paths()["purelib"]`), never derived by the case from a
//! layout constant, so a product that invents its own directory cannot satisfy
//! the assertions by agreeing with the test's arithmetic. Everything the case
//! asserts hangs off that one reported path.
//!
//! # Why the stub layout cannot pass
//!
//! Today `mamba sync` writes `<project>/.venv/site-packages/<module>/` with
//! `__init__.py`, `INSTALLER`, and `VERSION` markers, and `mamba run` makes it
//! importable by putting that directory on `PYTHONPATH`. Five assertions
//! separately refuse that shape:
//!
//! 1. `<purelib>/lib-1.0.dist-info/RECORD` exists and lists `lib/__init__.py`
//!    — a stub writes no dist-info anywhere, and the flat directory is not
//!    under `purelib`.
//! 2. `lib.__file__` resolves under `purelib` — the flat `site-packages` is a
//!    sibling of `lib/`, not a descendant of it.
//! 3. `lib.answer()` returns `42` — the wheel's own module body. A generated
//!    stub carries markers, not the packaged function.
//! 4. The child process sees no `PYTHONPATH` — the injection that makes the
//!    stub reachable is the thing being removed.
//! 5. `mamba pip list --format freeze` pins `lib==1.0` — the inventory reads
//!    dist-info, which only a real install writes.
//!
//! # Facets
//!
//! - **Behavior**: three lock origins — a frozen-index entry (`path`,
//!   `source_kind = "index"`), a `direct_file` entry (relative `path`), and a
//!   registry entry (`url` plus `sha256`, served locally) — reach the same
//!   installed, importable distribution; the second `sync` is the documented
//!   `no_op` and `sync --check` exits 0, so convergence is idempotent rather
//!   than a reinstall every run.
//! - **Security (fail-closed install inputs)**: an entry whose artifact is
//!   missing, an entry that names neither `path` nor `url`, and an artifact
//!   whose bytes do not match the locked `sha256` each abort the sync naming
//!   the package, and leave nothing installed. Silently fabricating a module
//!   for a package whose artifact was never seen is the fail-open state this
//!   work item removes: it would let a lock entry no artifact backs satisfy an
//!   import, and it is what the tree does today.
//! - **Performance**: the work item names no budget, so this case asserts no
//!   timing.
//!
//! # Hermeticity
//!
//! No network, ever: every wheel is built in-process, the index is a local
//! directory, and the one URL is served by an in-process `wiremock` server
//! bound to loopback. `HOME` and `MAMBA_CACHE_DIR` are pinned into the temp
//! tree; `MAMBA_FROZEN_INDEX`, `MAMBA_INDEX_URL`, `MAMBA_JOBS`,
//! `XDG_CACHE_HOME`, `VIRTUAL_ENV`, and `PYTHONPATH` are removed so no ambient
//! value can supply an answer. Nothing is skipped and nothing is `#[ignore]`d:
//! a missing `python3` fails the case naming it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mamba::pkgmanage::pkgmgr::wheel_build::{
    compose_filename, CoreMetadata, WheelBuilder, WheelMetadata,
};
use sha2::{Digest, Sha256};

/// The distribution under test: one module, one function, one version.
const DIST: &str = "lib";
const VERSION: &str = "1.0";
const MODULE: &str = "lib";
const DIST_INFO: &str = "lib-1.0.dist-info";
const WHEEL_FILE: &str = "lib-1.0-py3-none-any.whl";
/// The module body the wheel carries. `answer()` exists only in the packaged
/// source, so reading `42` back proves the wheel's own bytes were installed.
const MODULE_BODY: &str = "def answer():\n    return 42\n";

/// Printed by the probe below when the child process has no `PYTHONPATH`.
const NO_PYTHONPATH: &str = "<unset>";

/// Four lines: where `lib` came from, what it computes, whether the runner
/// injected an import path, and which interpreter is running.
const IMPORT_PROBE: &str = "import os, sys, lib\n\
                            print(lib.__file__)\n\
                            print(lib.answer())\n\
                            print(os.environ.get('PYTHONPATH', '<unset>'))\n\
                            print(sys.executable)\n";

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
fn expect_refusal(step: &str, args: &[&str], out: &Output) -> String {
    assert!(
        !out.status.success(),
        "step `{step}` must be refused, not accepted: {}",
        render(args, out)
    );
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

/// Build the real, importable wheel through the product's own wheel builder.
fn build_wheel(out_dir: &Path) -> PathBuf {
    let filename = compose_filename(DIST, VERSION, "py3", "none", "any");
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-sync-real-install");
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

fn sha256_file(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    sha256_bytes(&bytes)
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn canon(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// A TOML basic-string body for a filesystem path.
fn toml_str(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}

fn path_arg(path: &Path) -> String {
    path.to_str()
        .unwrap_or_else(|| panic!("fixture: path {} is not utf-8", path.display()))
        .to_string()
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
        panic!(
            "this case needs `python3` on PATH and found none in {:?}",
            path_var
        )
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

/// One scaffolded project: `mamba init`, then a real PEP 405 environment from
/// the host `python3`, plus the `purelib` that environment reports.
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
        let purelib = case.reported_purelib();
        Case { purelib, ..case }
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

    /// Ask the environment's own interpreter where a pure-Python distribution
    /// belongs. This is the case's only definition of "installed".
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
        let purelib = PathBuf::from(stdout_of(&out).trim());
        assert!(
            purelib.is_dir(),
            "fixture: the environment reported purelib {}, which is not a directory",
            purelib.display()
        );
        let venv = canon(&self.project.join(".venv"));
        assert!(
            canon(&purelib).starts_with(&venv),
            "fixture: the environment reported purelib {}, which is outside {}",
            purelib.display(),
            venv.display()
        );
        purelib
    }

    fn run(&self, args: &[&str]) -> Output {
        run(&self.project, &self.home, args)
    }

    fn write_lock(&self, body: &str) {
        let path = self.project.join("mamba.lock");
        std::fs::write(&path, body).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    }

    fn record(&self) -> PathBuf {
        self.purelib.join(DIST_INFO).join("RECORD")
    }

    fn installed_module(&self) -> PathBuf {
        self.purelib.join(MODULE).join("__init__.py")
    }

    /// The pre-change layout: a flat `site-packages` directly under the
    /// environment root, holding a generated module and its markers.
    fn stub_module(&self) -> PathBuf {
        self.project
            .join(".venv")
            .join("site-packages")
            .join(MODULE)
            .join("__init__.py")
    }

    /// Every observation the work item promises for one installed lock entry.
    /// `origin` names the sub-case so a failure says which lock shape broke.
    fn assert_wheel_reached_the_interpreter(&self, origin: &str) {
        let record = self.record();
        assert!(
            record.is_file(),
            "[{origin}] `mamba sync` must install {DIST}=={VERSION} into the environment's own \
             site-packages: {} is missing. Present under {}: {:?}",
            record.display(),
            self.purelib.display(),
            listing(&self.purelib),
        );
        let record_body = std::fs::read_to_string(&record)
            .unwrap_or_else(|e| panic!("read {}: {e}", record.display()));
        assert!(
            record_body.contains(&format!("{MODULE}/__init__.py")),
            "[{origin}] RECORD must inventory the installed module\n--- RECORD ---\n{record_body}"
        );
        assert!(
            self.installed_module().is_file(),
            "[{origin}] the wheel's module must land at {}",
            self.installed_module().display()
        );
        assert!(
            !self.stub_module().is_file(),
            "[{origin}] the flat stub layout must not be written: {} exists",
            self.stub_module().display()
        );

        // The import path, with nothing pointed at it by the runner.
        let args = ["run", "--", "python", "-c", IMPORT_PROBE];
        let out = self.run(&args);
        expect_ok(
            &format!("[{origin}] run -- python -c <import probe>"),
            &args,
            &out,
        );
        let stdout = stdout_of(&out);
        let lines: Vec<&str> = stdout.lines().collect();
        assert!(
            lines.len() >= 4,
            "[{origin}] the import probe must print four lines: {}",
            render(&args, &out)
        );

        let imported = canon(Path::new(lines[0]));
        assert!(
            imported.starts_with(canon(&self.purelib)),
            "[{origin}] `import {MODULE}` must resolve under the environment's purelib {}, got {}",
            canon(&self.purelib).display(),
            imported.display()
        );
        assert_eq!(
            lines[1], "42",
            "[{origin}] the imported module must be the wheel's own source, which defines \
             `answer()`; got {:?}",
            lines[1]
        );
        assert_eq!(
            lines[2], NO_PYTHONPATH,
            "[{origin}] `mamba run` must not put an import path on PYTHONPATH; got {:?}",
            lines[2]
        );
        // `.venv/bin/python` is a symlink to the base interpreter, so the
        // directory holding it is what identifies the environment;
        // canonicalizing the executable itself resolves out of the venv on
        // every host that seeds one from a version manager.
        let interpreter = Path::new(lines[3]);
        let interpreter_dir = canon(interpreter.parent().unwrap_or_else(|| Path::new("")));
        assert!(
            interpreter_dir.starts_with(canon(&self.project.join(".venv"))),
            "[{origin}] `mamba run -- python` must run the environment's own interpreter, got {}",
            interpreter.display()
        );

        // The inventory every pip-shaped tool reads.
        let args = ["pip", "list", "--format", "freeze"];
        let out = self.run(&args);
        expect_ok(&format!("[{origin}] pip list --format freeze"), &args, &out);
        assert!(
            stdout_of(&out).contains(&format!("{DIST}=={VERSION}")),
            "[{origin}] the environment inventory must pin {DIST}=={VERSION}: {}",
            render(&args, &out)
        );

        // Converged: the second run installs nothing and says so.
        let args = ["sync"];
        let out = self.run(&args);
        expect_ok(&format!("[{origin}] second sync"), &args, &out);
        assert!(
            stderr_of(&out).contains("no_op: environment already in sync with mamba.lock"),
            "[{origin}] the second `mamba sync` must be the documented no-op: {}",
            render(&args, &out)
        );

        let args = ["sync", "--check"];
        let out = self.run(&args);
        expect_ok(&format!("[{origin}] sync --check"), &args, &out);
        assert!(
            stdout_of(&out).contains("environment is synchronized with mamba.lock"),
            "[{origin}] `mamba sync --check` must report a synchronized environment: {}",
            render(&args, &out)
        );
    }

    /// Nothing the refused entry named may exist afterwards, in either layout.
    fn assert_nothing_installed(&self, origin: &str) {
        assert!(
            !self.purelib.join(DIST_INFO).exists(),
            "[{origin}] a refused entry must leave no distribution behind: {} exists",
            self.purelib.join(DIST_INFO).display()
        );
        assert!(
            !self.purelib.join(MODULE).exists(),
            "[{origin}] a refused entry must leave no module behind: {} exists",
            self.purelib.join(MODULE).display()
        );
        assert!(
            !self.stub_module().is_file(),
            "[{origin}] a refused entry must not be fabricated as a stub: {} exists",
            self.stub_module().display()
        );
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

/// Serve `body` at `url_path` from an in-process loopback server. The returned
/// runtime must stay alive for as long as the server is needed.
fn serve(url_path: &str, body: Vec<u8>) -> (tokio::runtime::Runtime, String) {
    let rt = tokio::runtime::Runtime::new().expect("build a runtime for the local artifact server");
    let base = rt.block_on(async {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(url_path.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(body))
            .mount(&server)
            .await;
        let uri = server.uri();
        std::mem::forget(server);
        uri
    });
    (rt, base)
}

/// A one-entry lock for a registry artifact: `url` plus `sha256`, no `path`.
fn registry_lock(url: &str, sha256: &str) -> String {
    format!(
        "format_version = 1\n\
         input_hash = \"e2e-sync-real-install\"\n\
         \n\
         [[package]]\n\
         name = \"{DIST}\"\n\
         version = \"{VERSION}\"\n\
         sha256 = \"{sha256}\"\n\
         url = \"{url}\"\n\
         source = \"pypi://{DIST}/{VERSION}\"\n\
         dependencies = []\n"
    )
}

#[test]
fn frozen_index_entry_installs_the_wheel_into_the_environment_and_imports_it() {
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

    // Fixture self-check: the entry this sub-case is about must be the one the
    // lock carries, so a resolver change fails here rather than as an install
    // assertion below.
    let lock = std::fs::read_to_string(case.project.join("mamba.lock")).expect("read mamba.lock");
    assert!(
        lock.contains("source_kind = \"index\"") && lock.contains("path = "),
        "fixture: `mamba add --index` must lock an index entry with a path\n--- mamba.lock ---\n{lock}"
    );

    let sync_args = ["sync"];
    let out = case.run(&sync_args);
    expect_ok("sync", &sync_args, &out);

    // The flat layout is retired, not merely unused: nothing writes it.
    assert!(
        !case.project.join(".venv").join("site-packages").exists(),
        "`mamba sync` must not write the flat layout beside the environment's own: {} exists",
        case.project.join(".venv").join("site-packages").display()
    );

    case.assert_wheel_reached_the_interpreter("index");
}

#[test]
fn direct_file_entry_installs_the_wheel_into_the_environment_and_imports_it() {
    let case = Case::start();
    let wheel = build_wheel(&case.project.join("wheels"));
    assert!(
        wheel.is_file(),
        "fixture: {} is not a file",
        wheel.display()
    );

    let add_args = ["add", "./wheels/lib-1.0-py3-none-any.whl"];
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

    case.assert_wheel_reached_the_interpreter("direct_file");
}

#[test]
fn url_and_sha256_entry_installs_the_downloaded_wheel_and_imports_it() {
    let case = Case::start();
    let wheel = build_wheel(&case.root.join("wheels"));
    let bytes = std::fs::read(&wheel).expect("read the built wheel");
    let digest = sha256_bytes(&bytes);

    let url_path = format!("/files/{WHEEL_FILE}");
    let (_server_rt, base) = serve(&url_path, bytes);
    case.write_lock(&registry_lock(&format!("{base}{url_path}"), &digest));

    let sync_args = ["sync"];
    let out = case.run(&sync_args);
    expect_ok("sync", &sync_args, &out);

    case.assert_wheel_reached_the_interpreter("url+sha256");
}

#[test]
fn sync_refuses_a_lock_entry_whose_artifact_is_missing() {
    let case = Case::start();
    let wheel = build_wheel(&case.root.join("wheels"));
    let digest = sha256_file(&wheel);
    let absent = case.root.join("absent").join(WHEEL_FILE);
    assert!(
        !absent.exists(),
        "fixture: {} must not exist",
        absent.display()
    );

    case.write_lock(&format!(
        "format_version = 1\n\
         input_hash = \"e2e-sync-real-install\"\n\
         \n\
         [[package]]\n\
         name = \"{DIST}\"\n\
         version = \"{VERSION}\"\n\
         sha256 = \"{digest}\"\n\
         url = \"\"\n\
         source_kind = \"index\"\n\
         path = \"{path}\"\n\
         dependencies = []\n",
        path = toml_str(&absent)
    ));

    let sync_args = ["sync"];
    let out = case.run(&sync_args);
    let said = expect_refusal("sync with a missing artifact", &sync_args, &out);
    assert!(
        said.contains(DIST),
        "the refusal must name the package: {}",
        render(&sync_args, &out)
    );
    assert!(
        said.contains(&absent.display().to_string()),
        "the refusal must name the artifact it could not read: {}",
        render(&sync_args, &out)
    );
    case.assert_nothing_installed("missing artifact");
}

#[test]
fn sync_refuses_a_lock_entry_that_names_neither_a_path_nor_a_url() {
    let case = Case::start();
    case.write_lock(&format!(
        "format_version = 1\n\
         input_hash = \"e2e-sync-real-install\"\n\
         \n\
         [[package]]\n\
         name = \"{DIST}\"\n\
         version = \"{VERSION}\"\n\
         sha256 = \"\"\n\
         url = \"\"\n\
         dependencies = []\n"
    ));

    let sync_args = ["sync"];
    let out = case.run(&sync_args);
    let said = expect_refusal("sync with an origin-less entry", &sync_args, &out);
    assert!(
        said.contains(DIST),
        "the refusal must name the package it cannot install: {}",
        render(&sync_args, &out)
    );
    case.assert_nothing_installed("no origin");
}

#[test]
fn sync_refuses_an_artifact_whose_bytes_do_not_match_the_locked_sha256() {
    let case = Case::start();
    let wheel = build_wheel(&case.root.join("wheels"));
    let locked_digest = sha256_file(&wheel);

    // The server answers with different bytes than the lock pins: the
    // supply-chain case the digest exists to refuse.
    let mut tampered = std::fs::read(&wheel).expect("read the built wheel");
    tampered.extend_from_slice(b"tampered");
    assert_ne!(
        sha256_bytes(&tampered),
        locked_digest,
        "fixture: the tampered body must not hash to the locked digest"
    );

    let url_path = format!("/files/{WHEEL_FILE}");
    let (_server_rt, base) = serve(&url_path, tampered);
    case.write_lock(&registry_lock(&format!("{base}{url_path}"), &locked_digest));

    let sync_args = ["sync"];
    let out = case.run(&sync_args);
    let said = expect_refusal("sync with a tampered artifact", &sync_args, &out);
    assert!(
        said.contains(DIST),
        "the refusal must name the package whose artifact failed verification: {}",
        render(&sync_args, &out)
    );
    case.assert_nothing_installed("sha256 mismatch");
}
