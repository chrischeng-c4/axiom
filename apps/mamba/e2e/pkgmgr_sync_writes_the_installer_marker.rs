//! Black-box contract: a distribution `mamba sync` installs carries PEP 376's
//! `INSTALLER` marker inside its own dist-info, and that marker is inventoried
//! in the dist-info's `RECORD`.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary over a throwaway directory tree
//! and reads only what the binary wrote or what the project's own interpreter
//! reports. Nothing installed is hand-written: the wheel is built by the
//! product's own `WheelBuilder`, frozen by `mamba index build`, locked by
//! `mamba add`, and installed by `mamba sync`, so a green run means those four
//! steps agree on one artifact. The harness is the one
//! `e2e/pkgmgr_sync_real_install.rs` established; only the observations are
//! new. That the wheel lands at all stays that case's contract, and is a
//! fixture self-check here.
//!
//! # The observation point
//!
//! `purelib` is asked of the `.venv` interpreter itself
//! (`sysconfig.get_paths()["purelib"]`), never derived by the case from a
//! layout constant, and the marker is read back a second time through that
//! same interpreter's `importlib.metadata.distribution("lib")
//! .read_text("INSTALLER")`. A product that writes the six bytes somewhere its
//! own interpreter cannot see them therefore fails, even though a file exists
//! on disk.
//!
//! # Why the tree fails this today
//!
//! `Installer::install` extracts the wheel, places its files, writes the
//! console-script wrappers and their rows, then renders `RECORD` -- and writes
//! nothing else into the installed dist-info. No `INSTALLER` is created
//! anywhere under `purelib`, `read_text("INSTALLER")` returns `None`, and
//! `RECORD` carries no row for it. Both cases below fail on the missing file.
//!
//! # Facets
//!
//! - **Behavior**: after `mamba sync` the marker holds exactly `mamba` and one
//!   newline -- six bytes, the `pip`/`uv` convention, no version and no
//!   timestamp -- at `<purelib>/lib-1.0.dist-info/INSTALLER`; `RECORD` carries
//!   the row for it beside the rows it already had; and the environment's own
//!   interpreter reads those bytes back. A second `mamba sync` is still the
//!   documented `no_op` and leaves both files byte-identical, so the marker
//!   neither defeats the already-installed fast path nor is rewritten on every
//!   convergence.
//! - **Security (provenance and inventory completeness)**: `INSTALLER` is the
//!   provenance record an operator and every other installer read to learn
//!   which tool owns a distribution -- pip refuses to uninstall what it did not
//!   install by reading exactly this file -- so an absent or mislabelled marker
//!   leaves mamba's installs indistinguishable from another tool's.
//!   Meanwhile `RECORD` is that dist-info's integrity inventory and the
//!   uninstall walk's only map: a file inside the dist-info that `RECORD` does
//!   not list is covered by neither. Case one therefore requires the row to
//!   appear exactly once, to carry a real digest rather than the blanked shape
//!   the `RECORD` self-row takes, and -- recomputing the digest from the bytes
//!   on disk -- to describe the file it names. It also requires the marker to
//!   live only inside a `.dist-info` directory, so a provenance claim cannot be
//!   planted at the root of `purelib`, inside the imported package, or beside
//!   the environment's scripts, where an audit of some other distribution would
//!   read it as its own.
//! - **Performance**: the work item names no budget, so this case asserts no
//!   timing.
//!
//! # Case two is deliberately narrower
//!
//! Case two observes the marker file, the `no_op` and the byte-identity, and
//! never the `RECORD` row. That is what makes the work item's negative control
//! discriminating: deleting the row's `RecordEntry` push while leaving the file
//! write in place must go red on case one alone. A row assertion here would
//! turn both cases red together and hide which half the product lost.
//!
//! # Hermeticity
//!
//! No network, ever: the wheel is built in-process and the index is a local
//! directory. `HOME` and `MAMBA_CACHE_DIR` are pinned into the temp tree;
//! `MAMBA_FROZEN_INDEX`, `MAMBA_INDEX_URL`, `MAMBA_JOBS`, `XDG_CACHE_HOME`,
//! `VIRTUAL_ENV` and `PYTHONPATH` are removed so no ambient value can supply an
//! answer. Nothing is skipped and nothing is `#[ignore]`d: a missing `python3`
//! fails the case naming it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use base64::Engine;
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
/// The module body the wheel carries; its row in `RECORD` is what proves the
/// marker was added to the existing inventory rather than replacing it.
const MODULE_BODY: &str = "def answer():\n    return 42\n";

/// PEP 376's marker names the installing tool and nothing else: pip writes
/// `pip\n`, uv writes `uv\n`.
const INSTALLER_BYTES: &[u8] = b"mamba\n";

/// The `RECORD` row those six bytes must produce, in the shape every other row
/// takes. The digest is fixed because the content is fixed; the case also
/// recomputes it from the bytes on disk rather than trusting this constant
/// alone.
const INSTALLER_ROW: &str =
    "lib-1.0.dist-info/INSTALLER,sha256=Kp3AVDOCkkOUcOc4jNDrHOwS2eECHeFZCXmKLfmRhDc,6";

/// What the environment's own interpreter must print for the marker. `repr`
/// separates the missing marker (`None`) from an empty one (`''`) and shows the
/// trailing newline instead of eating it.
const INSTALLER_REPR: &str = "'mamba\\n'";

/// One line: what `importlib.metadata` reads out of the installed dist-info.
const METADATA_PROBE: &str = "import importlib.metadata as md\n\
                              print(repr(md.distribution('lib').read_text('INSTALLER')))\n";

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

/// Build the real, importable wheel through the product's own wheel builder.
fn build_wheel(out_dir: &Path) -> PathBuf {
    let filename = compose_filename(DIST, VERSION, "py3", "none", "any");
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-sync-installer-marker");
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

/// The URL-safe, unpadded base64 of a sha256 -- the encoding `RECORD` rows use.
fn b64url_sha256(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(bytes))
}

fn read_bytes(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
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

/// Every file named `INSTALLER` anywhere under `dir`. Used to refuse a marker
/// written outside a distribution's own dist-info; the environment's seeded
/// distributions carry their own markers, which is why the assertion is on each
/// marker's owning directory rather than on the count.
fn markers_under(dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(next) = stack.pop() {
        let entries = match std::fs::read_dir(&next) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.file_name().and_then(|n| n.to_str()) == Some("INSTALLER") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

fn canon(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// The `python3` this host offers, resolved to the interpreter it actually
/// runs. `python3` on `PATH` is commonly a version-manager shim whose own
/// resolution depends on `HOME`, and this case pins `HOME` into a temp tree;
/// asking the shim for `sys.executable` gets the real interpreter, which
/// `mamba venv --python` can seed from under any environment. A host with no
/// `python3` fails here, naming it -- the case never skips.
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

/// One scaffolded project: `mamba init`, a real PEP 405 environment from the
/// host `python3`, and the `purelib` that environment reports.
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
        let stdout = self.ask_interpreter(
            "purelib",
            "import sysconfig; print(sysconfig.get_paths()['purelib'])",
        );
        let purelib = PathBuf::from(stdout.trim());
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

    /// Run a snippet on the environment's own interpreter, with no import path
    /// and no activation handed to it.
    fn ask_interpreter(&self, step: &str, snippet: &str) -> String {
        let python = self.venv_python();
        let out = Command::new(&python)
            .args(["-c", snippet])
            .current_dir(&self.project)
            .env("HOME", &self.home)
            .env_remove("PYTHONPATH")
            .env_remove("VIRTUAL_ENV")
            .output()
            .unwrap_or_else(|e| panic!("spawn {}: {e}", python.display()));
        assert!(
            out.status.success(),
            "`{} -c <{step}>` failed: {}",
            python.display(),
            String::from_utf8_lossy(&out.stderr)
        );
        stdout_of(&out)
    }

    fn run(&self, args: &[&str]) -> Output {
        run(&self.project, &self.home, args)
    }

    fn dist_info(&self) -> PathBuf {
        self.purelib.join(DIST_INFO)
    }

    fn installer_path(&self) -> PathBuf {
        self.dist_info().join("INSTALLER")
    }

    fn record_path(&self) -> PathBuf {
        self.dist_info().join("RECORD")
    }

    fn read_record(&self) -> String {
        let path = self.record_path();
        String::from_utf8_lossy(&read_bytes(&path)).to_string()
    }

    /// What `importlib.metadata` on the environment's own interpreter reports
    /// for the marker, as a Python `repr`.
    fn marker_as_the_interpreter_reads_it(&self) -> String {
        self.ask_interpreter("importlib.metadata INSTALLER", METADATA_PROBE)
            .trim()
            .to_string()
    }

    /// Build, freeze, lock and install `lib-1.0` -- the four product steps the
    /// committed real-install case established. Everything the two cases below
    /// observe was written by the `mamba sync` at the end of this method.
    fn install_lib(&self) {
        let wheel = build_wheel(&self.root.join("wheels"));
        let index = self.root.join("index");

        let index_arg = path_arg(&index);
        let wheel_arg = path_arg(&wheel);
        let build_args = [
            "index",
            "build",
            "--out",
            index_arg.as_str(),
            wheel_arg.as_str(),
        ];
        let out = self.run(&build_args);
        expect_ok("index build", &build_args, &out);

        let add_args = ["add", DIST, "--index", index_arg.as_str()];
        let out = self.run(&add_args);
        expect_ok("add --index", &add_args, &out);

        // Fixture self-check: a resolver change must fail here, naming the
        // lock, rather than as a missing marker below.
        let lock =
            std::fs::read_to_string(self.project.join("mamba.lock")).expect("read mamba.lock");
        assert!(
            lock.contains("source_kind = \"index\"") && lock.contains("path = "),
            "fixture: `mamba add --index` must lock an index entry with a path\n\
             --- mamba.lock ---\n{lock}"
        );

        let sync_args = ["sync"];
        let out = self.run(&sync_args);
        expect_ok("sync", &sync_args, &out);

        // Fixture self-check: that the wheel lands at all is
        // `pkgmgr_sync_real_install`'s contract, not this case's. If the
        // install stops happening, say so here instead of blaming the marker.
        assert!(
            self.record_path().is_file(),
            "fixture: `mamba sync` must install {DIST}=={VERSION} into the environment's own \
             site-packages: {} is missing. Present under {}: {:?}",
            self.record_path().display(),
            self.purelib.display(),
            listing(&self.purelib),
        );
    }
}

/// Case one: the marker exists, holds exactly `mamba\n`, is inventoried in
/// `RECORD` by a row that describes its own bytes, is read back by the
/// environment's own interpreter, and lives nowhere but a dist-info.
#[test]
fn sync_writes_the_installer_marker_and_inventories_it_in_record() {
    let case = Case::start();
    case.install_lib();

    let installer = case.installer_path();
    assert!(
        installer.is_file(),
        "`mamba sync` must write PEP 376's INSTALLER marker into the installed dist-info: {} is \
         missing. Present in {}: {:?}",
        installer.display(),
        case.dist_info().display(),
        listing(&case.dist_info()),
    );

    let bytes = read_bytes(&installer);
    assert_eq!(
        bytes,
        INSTALLER_BYTES,
        "the marker must name the installing tool and nothing else -- no version, no path, no \
         timestamp. Expected {:?}, got {:?}",
        String::from_utf8_lossy(INSTALLER_BYTES),
        String::from_utf8_lossy(&bytes),
    );

    // RECORD is the dist-info's integrity inventory and the uninstall walk's
    // only map: exactly one row, in the shape every other row takes.
    let record_body = case.read_record();
    let prefix = format!("{DIST_INFO}/INSTALLER,");
    let rows: Vec<&str> = record_body
        .lines()
        .filter(|line| line.starts_with(prefix.as_str()))
        .collect();
    assert_eq!(
        rows.len(),
        1,
        "RECORD must inventory the marker exactly once: an unlisted file is covered by neither \
         the integrity inventory nor the uninstall walk, and a second row means a second writer \
         appended to RECORD\n--- RECORD ---\n{record_body}"
    );
    assert_eq!(
        rows[0], INSTALLER_ROW,
        "the marker's RECORD row must carry the digest and byte length of the marker, not the \
         blanked fields the RECORD self-row takes\n--- RECORD ---\n{record_body}"
    );

    // The constant is not taken on trust: the recorded digest and size must
    // describe the bytes that are actually on disk.
    let observed_row = format!(
        "{DIST_INFO}/INSTALLER,sha256={},{}",
        b64url_sha256(&bytes),
        bytes.len()
    );
    assert_eq!(
        observed_row, INSTALLER_ROW,
        "the RECORD row must describe the marker's own bytes"
    );

    // The rows the committed cases observe survive: the marker is one more
    // entry rendered by the same writer, not a second pass over RECORD.
    assert!(
        record_body.contains(&format!("{MODULE}/__init__.py,sha256=")),
        "RECORD must still inventory the installed module beside the marker\n\
         --- RECORD ---\n{record_body}"
    );

    // The environment's own metadata reader, not the case's path arithmetic.
    let seen = case.marker_as_the_interpreter_reads_it();
    assert_eq!(
        seen, INSTALLER_REPR,
        "the environment's own interpreter must read the marker back through \
         importlib.metadata.distribution(\"{DIST}\").read_text(\"INSTALLER\"); got {seen}"
    );

    // Fail-closed placement: the provenance claim belongs to one distribution,
    // so it may exist only inside that distribution's dist-info. A marker at
    // the root of purelib, inside the imported package, or beside the
    // environment's scripts would be read as an answer for something this sync
    // never installed.
    for stray in [
        case.project.join("INSTALLER"),
        case.project.join(".venv").join("INSTALLER"),
        case.project.join(".venv").join("bin").join("INSTALLER"),
        case.project
            .join(".venv")
            .join("site-packages")
            .join("INSTALLER"),
    ] {
        assert!(
            !stray.exists(),
            "the marker must live only in the installed dist-info: {} exists",
            stray.display()
        );
    }
    let markers = markers_under(&case.purelib);
    assert!(
        markers.contains(&installer),
        "sweep self-check: {} must be among the markers found under {}: {:?}",
        installer.display(),
        case.purelib.display(),
        markers
    );
    for marker in &markers {
        let owner = marker
            .parent()
            .and_then(Path::file_name)
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        assert!(
            owner.ends_with(".dist-info"),
            "an INSTALLER marker may only live inside a distribution's dist-info; found {}",
            marker.display()
        );
    }
}

/// Case two: converging again changes nothing. The marker must not defeat the
/// already-installed fast path, and must not be rewritten on every run.
///
/// Deliberately silent about the `RECORD` row -- see the module header.
#[test]
fn a_second_sync_is_a_no_op_that_leaves_the_marker_byte_identical() {
    let case = Case::start();
    case.install_lib();

    let installer = case.installer_path();
    assert!(
        installer.is_file(),
        "the first `mamba sync` must have written PEP 376's INSTALLER marker: {} is missing. \
         Present in {}: {:?}",
        installer.display(),
        case.dist_info().display(),
        listing(&case.dist_info()),
    );
    let installer_before = read_bytes(&installer);
    assert_eq!(
        installer_before,
        INSTALLER_BYTES,
        "the marker the first sync wrote must already be {:?}, got {:?}",
        String::from_utf8_lossy(INSTALLER_BYTES),
        String::from_utf8_lossy(&installer_before),
    );
    let record_before = read_bytes(&case.record_path());

    let sync_args = ["sync"];
    let out = case.run(&sync_args);
    expect_ok("second sync", &sync_args, &out);
    assert!(
        stderr_of(&out).contains("no_op: environment already in sync with mamba.lock"),
        "the marker must not defeat the already-installed fast path: the second `mamba sync` must \
         still be the documented no-op: {}",
        render(&sync_args, &out)
    );

    let installer_after = read_bytes(&installer);
    assert_eq!(
        installer_after,
        installer_before,
        "a converged `mamba sync` must leave the marker byte-identical: {:?} became {:?}",
        String::from_utf8_lossy(&installer_before),
        String::from_utf8_lossy(&installer_after),
    );
    let record_after = read_bytes(&case.record_path());
    assert_eq!(
        String::from_utf8_lossy(&record_after),
        String::from_utf8_lossy(&record_before),
        "a converged `mamba sync` must leave RECORD byte-identical"
    );

    // Still readable through the environment's own metadata reader afterwards.
    let seen = case.marker_as_the_interpreter_reads_it();
    assert_eq!(
        seen, INSTALLER_REPR,
        "after the second sync the interpreter must still read the marker; got {seen}"
    );

    // And the environment still reports itself converged.
    let check_args = ["sync", "--check"];
    let out = case.run(&check_args);
    expect_ok("sync --check", &check_args, &out);
    assert!(
        stdout_of(&out).contains("environment is synchronized with mamba.lock"),
        "`mamba sync --check` must report a synchronized environment: {}",
        render(&check_args, &out)
    );
}
