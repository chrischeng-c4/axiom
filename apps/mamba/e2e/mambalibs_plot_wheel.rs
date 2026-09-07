//! Black-box contract: `mambalibs-plotkit` ships as a PyO3 `abi3` wheel that
//! the mamba package manager can freeze, resolve, install, and import as
//! `mambalibs.plot` — *beside* the already-shipping `mambalibs.array` and
//! `mambalibs.sci`, in one environment, out of one index.
//!
//! # What this case owns
//!
//! The case drives real tools over a throwaway tree and reads only what they
//! wrote. It links no product type: three wheels are built by `maturin` from
//! `apps/mamba/mambalibs/arraykit/pyo3`, `apps/mamba/mambalibs/scikit/pyo3`,
//! and `apps/mamba/mambalibs/plotkit/pyo3`, frozen together by one `mamba
//! index build`, locked by one `mamba add --index` naming all three
//! distributions, installed by one `mamba sync`, and finally judged by the
//! project interpreter itself. A green run means those five steps agree on
//! three artifacts that share a namespace — a wrong ABI tag, a wheel that
//! lands outside the environment, a wheel that ships `mambalibs/__init__.py`
//! and shadows its siblings, or an import that resolves from the source tree
//! each fails one named step.
//!
//! This is the last wheel of the `mamba@0.1.0` Milestone, so the coexistence
//! claim is the whole set rather than a pair: two kits sharing a namespace is
//! a fact about two wheels, and nothing about two proves the third does not
//! capture the directory when it arrives.
//!
//! # The observation point
//!
//! The environment's interpreter is asked where each module came from
//! (`mambalibs.array.__file__`, `mambalibs.sci.__file__`, and
//! `mambalibs.plot.__file__`), and those paths are compared against this
//! case's own `.venv` and against each other. Nothing is derived from a
//! layout constant, so an implementation that invents its own directory
//! cannot satisfy the assertions by agreeing with the test's arithmetic.
//!
//! # Steps, in order
//!
//! 1. `maturin build` in the arraykit crate produces exactly one
//!    `mambalibs_arraykit-*` wheel. This is a sibling the new wheel has to
//!    coexist with, so it is built here from source rather than assumed.
//! 2. `maturin build` in the scikit crate leaves a second wheel in the same
//!    directory: exactly one `mambalibs_scikit-*`, the other sibling.
//! 3. `maturin build` in the plotkit crate leaves a third: exactly one
//!    `mambalibs_plotkit-*`, tagged `cp312-abi3`, with both earlier wheels
//!    still beside it. Ordering the builds array → sci → plot is deliberate:
//!    the two landed wheels must build before the new one is asked for, so a
//!    failure here is about plotkit and not about the tools.
//! 4. `mamba index build` freezes all three wheels into one local index.
//! 5. `mamba init` scaffolds a PEP 621 `pyproject.toml`; `mamba venv
//!    --system-site-packages` builds the environment on `python3.12`.
//! 6. `mamba add mambalibs-arraykit mambalibs-scikit mambalibs-plotkit --index
//!    <dir>` locks all three index entries in one call, so none can come from
//!    anywhere else.
//! 7. `mamba sync` installs them.
//! 8. `python -c "import mambalibs.array, mambalibs.sci, mambalibs.plot"`
//!    prints all three `__file__`s: each is under this case's `.venv`, they
//!    share one `mambalibs/` directory, and that directory holds no
//!    `__init__.py` (PEP 420).
//! 9. `python -m pytest -p no:cacheprovider` over `e2e/mambalibs_plot/`
//!    collects at least one test and exits 0. Those files are the wheel's
//!    Python API contract over `plotkit`'s `viz` module.
//!
//! # Facets
//!
//! - **Behavior**: the nine steps above. The wheel is the only source of
//!   `mambalibs.plot`: the case asserts `__file__` is under its own `.venv`,
//!   so an import satisfied by the source tree, by `target/`, or by an ambient
//!   install fails rather than passes. Coexistence is asserted twice over —
//!   once on the artifacts (three wheels, one index, one `mamba add`) and once
//!   on the installed result (one shared namespace directory, no
//!   `__init__.py`) — because a wheel that captures the namespace breaks its
//!   siblings silently and would otherwise read as success.
//! - **Security (fail-closed inputs)**: the environment is stripped of
//!   `MAMBA_FROZEN_INDEX`, `MAMBA_INDEX_URL`, `MAMBA_PYTHON_TAG`,
//!   `VIRTUAL_ENV`, `PYTHONPATH`, and `XDG_CACHE_HOME`, and `HOME` and
//!   `MAMBA_CACHE_DIR` are pinned inside the temp tree, so no ambient value
//!   can supply an answer the case then credits to the product. The only index
//!   is the one the case just built: nothing is fetched from PyPI, and
//!   `MAMBA_PYTHON_TAG` is never set, so the `cp312-abi3` wheel has to be
//!   accepted by the shipped tag selector rather than by a loosened one.
//! - **Performance**: the work item names no budget, so this case asserts no
//!   timing.
//!
//! # Hermeticity, and why nothing is skipped
//!
//! No Python package is downloaded: `pytest` is the host interpreter's,
//! reached through `mamba venv --system-site-packages`, and the only index is
//! local. A host missing `maturin`, `python3.12`, or `pytest` fails the case
//! with a message naming the missing tool — there is no `#[ignore]`, no early
//! return, and no print-and-pass, because a case that goes green on an
//! unprepared host would report the toolchain instead of the product. Nothing
//! is written outside the temp tree: the Python contract judges returned
//! strings, never a file the case would then have to clean up.
//!
//! # Relationship to the two landed wheel cases
//!
//! `mambalibs_array_wheel.rs` and `mambalibs_sci_wheel.rs` are landed evidence
//! and are not edited by this one. The helpers below are copied rather than
//! shared: three cases reaching through one helper stop being independent, and
//! each kit's contract has to keep failing for its own reasons alone.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// `CARGO_MANIFEST_DIR` is `apps/mamba`, so the PyO3 crates and the pytest
/// directory are found without knowing where the repository is checked out.
fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn arraykit_crate_dir() -> PathBuf {
    manifest_dir().join("mambalibs").join("arraykit").join("pyo3")
}

fn scikit_crate_dir() -> PathBuf {
    manifest_dir().join("mambalibs").join("scikit").join("pyo3")
}

fn plotkit_crate_dir() -> PathBuf {
    manifest_dir().join("mambalibs").join("plotkit").join("pyo3")
}

fn pytest_dir() -> PathBuf {
    manifest_dir().join("e2e").join("mambalibs_plot")
}

/// The binary under test. `CARGO_BIN_EXE_*` is resolved by cargo at compile
/// time, so a missing binary is a build failure rather than a skipped case.
fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

fn render(program: &str, args: &[&str], out: &Output) -> String {
    format!(
        "`{program} {}` exited {:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        args.join(" "),
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    )
}

fn expect_ok(step: &str, program: &str, args: &[&str], out: &Output) {
    assert!(
        out.status.success(),
        "step `{step}` must succeed: {}",
        render(program, args, out)
    );
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn path_arg(path: &Path) -> String {
    path.to_str()
        .unwrap_or_else(|| panic!("fixture: path {} is not utf-8", path.display()))
        .to_string()
}

fn file_name_of(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_else(|| panic!("fixture: file name is not utf-8: {}", path.display()))
        .to_string()
}

/// The first `name` on `PATH`, or a panic naming the tool. This is the
/// "fail naming the missing tool" rule: the case never skips.
fn tool_on_path(name: &str, why: &str) -> PathBuf {
    let path_var = std::env::var_os("PATH")
        .unwrap_or_else(|| panic!("this case needs `{name}` ({why}), and PATH is unset"));
    for dir in std::env::split_paths(&path_var) {
        for candidate in [dir.join(name), dir.join(format!("{name}.exe"))] {
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    panic!(
        "this case needs `{name}` on PATH ({why}) and found none.\n\
         PATH = {path_var:?}\n\
         The contract is a prepared host: install `{name}` rather than skipping this case."
    )
}

/// `python3.12`, resolved to the interpreter it actually runs. `python3.12` on
/// PATH is commonly a version-manager shim whose own resolution depends on
/// `HOME`, and this case pins `HOME` into a temp tree; asking the shim for
/// `sys.executable` gets the real interpreter, which `mamba venv --python` can
/// seed from under any environment.
fn resolve_python312() -> PathBuf {
    let shim = tool_on_path("python3.12", "the project's oracle interpreter");
    let out = Command::new(&shim)
        .args(["-c", "import sys; print(sys.executable)"])
        .output()
        .unwrap_or_else(|e| panic!("spawn {}: {e}", shim.display()));
    assert!(
        out.status.success(),
        "`{} -c 'print(sys.executable)'` failed: {}",
        shim.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    let real = PathBuf::from(stdout_of(&out).trim());
    assert!(
        real.is_file(),
        "`{}` reported the interpreter {}, which is not a file",
        shim.display(),
        real.display()
    );
    real
}

/// The host interpreter must be able to `import pytest`: step 9 runs it
/// through `--system-site-packages`, and it is never fetched from an index.
fn require_pytest(python: &Path) {
    let out = Command::new(python)
        .args(["-c", "import pytest; print(pytest.__version__)"])
        .output()
        .unwrap_or_else(|e| panic!("spawn {}: {e}", python.display()));
    assert!(
        out.status.success(),
        "this case needs `pytest` importable by {} and it is not.\n{}\n\
         pytest is reached through `mamba venv --system-site-packages` and is never \
         fetched from an index: install it into that interpreter rather than skipping \
         this case.",
        python.display(),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Run the binary in `cwd` with an environment that cannot reach a registry, a
/// user-global cache, or an ambient import path. `MAMBA_PYTHON_TAG` is removed
/// as well: all three `cp312-abi3` wheels must be accepted by the shipped
/// selector.
fn run_mamba(cwd: &Path, home: &Path, args: &[&str]) -> Output {
    Command::new(mamba_bin())
        .args(args)
        .current_dir(cwd)
        .env("HOME", home)
        .env("MAMBA_CACHE_DIR", home.join("cache"))
        .env_remove("MAMBA_FROZEN_INDEX")
        .env_remove("MAMBA_INDEX_URL")
        .env_remove("MAMBA_JOBS")
        .env_remove("MAMBA_PYTHON_TAG")
        .env_remove("XDG_CACHE_HOME")
        .env_remove("VIRTUAL_ENV")
        .env_remove("PYTHONPATH")
        .output()
        .unwrap_or_else(|e| panic!("spawn {} {args:?}: {e}", mamba_bin().display()))
}

/// Every `.whl` directly under `dir`, sorted.
fn wheels_in(dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("whl"))
                .collect()
        })
        .unwrap_or_default();
    found.sort();
    found
}

/// Build one PyO3 crate into `dist` and return the single wheel there whose
/// name starts with `prefix`.
///
/// `VIRTUAL_ENV` and `PYTHONPATH` are removed so maturin builds for the
/// interpreter this case named, and not for whatever environment the developer
/// running it happened to have active.
fn build_wheel(
    maturin: &Path,
    crate_dir: &Path,
    dist: &Path,
    interpreter: &str,
    prefix: &str,
) -> PathBuf {
    let dist_arg = path_arg(dist);
    let args = [
        "build",
        "--out",
        dist_arg.as_str(),
        "--interpreter",
        interpreter,
    ];
    let out = Command::new(maturin)
        .args(args)
        .current_dir(crate_dir)
        .env_remove("VIRTUAL_ENV")
        .env_remove("PYTHONPATH")
        .output()
        .unwrap_or_else(|e| panic!("spawn {} {args:?}: {e}", maturin.display()));
    expect_ok(
        &format!("maturin build in {}", crate_dir.display()),
        "maturin",
        &args,
        &out,
    );

    let present = wheels_in(dist);
    let names: Vec<String> = present.iter().map(|p| file_name_of(p)).collect();
    let matching: Vec<PathBuf> = present
        .into_iter()
        .filter(|p| file_name_of(p).starts_with(prefix))
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "`maturin build` in {} must leave exactly one `{prefix}*` wheel in {}, and {} \
         of the {} wheel(s) there match: {names:?}\n{}",
        crate_dir.display(),
        dist.display(),
        matching.len(),
        names.len(),
        render("maturin", &args, &out)
    );
    matching.into_iter().next().expect("one matching wheel")
}

#[test]
fn plotkit_ships_as_an_abi3_wheel_that_coexists_with_the_array_and_science_kits() {
    let python312 = resolve_python312();
    require_pytest(&python312);
    let maturin = tool_on_path("maturin", "it builds the PyO3 wheels this case installs");

    let tmp = tempfile::tempdir().expect("create temp root");
    let root = tmp.path().to_path_buf();
    let dist = root.join("dist");
    let index = root.join("index");
    let project = root.join("project");
    let home = root.join("home");
    for dir in [&dist, &project, &home] {
        std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("create {}: {e}", dir.display()));
    }
    let interpreter_arg = path_arg(&python312);

    // --- step 1: the first sibling wheel, built from source ---------------
    let arraykit_dir = arraykit_crate_dir();
    assert!(
        arraykit_dir.join("Cargo.toml").is_file(),
        "step `maturin build` needs the landed arraykit crate at {}, and it does not exist. \
         This case installs `mambalibs-arraykit` beside the new wheel to show the kits share \
         one namespace, so the sibling is built here rather than assumed.",
        arraykit_dir.display()
    );
    let array_wheel = build_wheel(
        &maturin,
        &arraykit_dir,
        &dist,
        &interpreter_arg,
        "mambalibs_arraykit-",
    );

    // --- step 2: the second sibling wheel ---------------------------------
    let scikit_dir = scikit_crate_dir();
    assert!(
        scikit_dir.join("Cargo.toml").is_file(),
        "step `maturin build` needs the landed scikit crate at {}, and it does not exist. \
         This case installs `mambalibs-scikit` beside the new wheel to show the kits share \
         one namespace, so the sibling is built here rather than assumed.",
        scikit_dir.display()
    );
    let sci_wheel = build_wheel(
        &maturin,
        &scikit_dir,
        &dist,
        &interpreter_arg,
        "mambalibs_scikit-",
    );

    // --- step 3: the wheel under contract ---------------------------------
    let plotkit_dir = plotkit_crate_dir();
    assert!(
        plotkit_dir.join("Cargo.toml").is_file(),
        "step `maturin build` needs the PyO3 crate at {}, and it does not exist. \
         `mambalibs-plotkit` has no wheel to build until that crate ships: a cdylib bound \
         with pyo3's abi3-py312 feature, depending on `plotkit` with its default features \
         so the `viz` module is wrapped, and a pyproject.toml whose [tool.maturin] puts the \
         module at `mambalibs.plot` with no `mambalibs/__init__.py`. The arraykit and scikit \
         wheels above already built, so the toolchain is not what failed here.",
        plotkit_dir.display()
    );
    let plot_wheel = build_wheel(
        &maturin,
        &plotkit_dir,
        &dist,
        &interpreter_arg,
        "mambalibs_plotkit-",
    );

    let plot_wheel_name = file_name_of(&plot_wheel);
    assert!(
        plot_wheel_name.contains("cp312-abi3"),
        "the wheel's tags must contain `cp312-abi3` (the frozen ABI decision), \
         got {plot_wheel_name}"
    );
    let built = wheels_in(&dist);
    assert_eq!(
        built.len(),
        3,
        "the three builds must leave exactly three wheels in {}, found {:?}",
        dist.display(),
        built.iter().map(|p| file_name_of(p)).collect::<Vec<_>>()
    );
    for sibling in [&array_wheel, &sci_wheel] {
        assert!(
            sibling.is_file(),
            "building the plotkit wheel must not disturb the wheels beside it; {} is gone",
            sibling.display()
        );
    }

    // --- step 4: one index over all three wheels --------------------------
    let index_arg = path_arg(&index);
    let array_wheel_arg = path_arg(&array_wheel);
    let sci_wheel_arg = path_arg(&sci_wheel);
    let plot_wheel_arg = path_arg(&plot_wheel);
    let args = [
        "index",
        "build",
        "--out",
        index_arg.as_str(),
        array_wheel_arg.as_str(),
        sci_wheel_arg.as_str(),
        plot_wheel_arg.as_str(),
    ];
    let out = run_mamba(&project, &home, &args);
    expect_ok("mamba index build", "mamba", &args, &out);

    // --- step 5: mamba init + mamba venv ----------------------------------
    let args = ["init"];
    let out = run_mamba(&project, &home, &args);
    expect_ok("mamba init", "mamba", &args, &out);
    assert!(
        project.join("pyproject.toml").is_file(),
        "fixture: `mamba init` wrote no PEP 621 pyproject.toml in {}",
        project.display()
    );

    let args = [
        "venv",
        "--python",
        interpreter_arg.as_str(),
        "--system-site-packages",
    ];
    let out = run_mamba(&project, &home, &args);
    expect_ok("mamba venv --system-site-packages", "mamba", &args, &out);

    // --- step 6: mamba add, all three distributions, one index ------------
    let args = [
        "add",
        "mambalibs-arraykit",
        "mambalibs-scikit",
        "mambalibs-plotkit",
        "--index",
        index_arg.as_str(),
    ];
    let out = run_mamba(&project, &home, &args);
    expect_ok("mamba add --index", "mamba", &args, &out);

    // --- step 7: mamba sync -----------------------------------------------
    let args = ["sync"];
    let out = run_mamba(&project, &home, &args);
    expect_ok("mamba sync", "mamba", &args, &out);

    // --- step 8: import, from the environment and nowhere else ------------
    let venv = project.join(".venv");
    let venv_python = [
        venv.join("bin").join("python"),
        venv.join("Scripts").join("python.exe"),
    ]
    .into_iter()
    .find(|p| p.is_file())
    .unwrap_or_else(|| {
        panic!(
            "fixture: `mamba venv` left no interpreter under {}",
            venv.display()
        )
    });

    let import_args = [
        "-c",
        "import mambalibs.array as a\nimport mambalibs.sci as s\nimport mambalibs.plot as p\n\
         print(a.__file__)\nprint(s.__file__)\nprint(p.__file__)",
    ];
    let out = Command::new(&venv_python)
        .args(import_args)
        .current_dir(&project)
        .env("HOME", &home)
        .env_remove("PYTHONPATH")
        .env_remove("VIRTUAL_ENV")
        .output()
        .unwrap_or_else(|e| panic!("spawn {} {import_args:?}: {e}", venv_python.display()));
    expect_ok(
        "import mambalibs.array, mambalibs.sci, mambalibs.plot",
        "python",
        &import_args,
        &out,
    );

    let printed = stdout_of(&out);
    let lines: Vec<&str> = printed
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let modules = ["mambalibs.array", "mambalibs.sci", "mambalibs.plot"];
    assert_eq!(
        lines.len(),
        modules.len(),
        "step `import mambalibs.array, mambalibs.sci, mambalibs.plot` must print one \
         `__file__` per module: {}",
        render("python", &import_args, &out)
    );

    let venv_real = std::fs::canonicalize(&venv).unwrap_or_else(|_| venv.clone());
    let mut resolved: Vec<PathBuf> = Vec::new();
    for (module, line) in modules.iter().zip(&lines) {
        let module_file = PathBuf::from(line);
        let module_real = std::fs::canonicalize(&module_file).unwrap_or_else(|e| {
            panic!(
                "`{module}.__file__` is {} and cannot be canonicalized: {e}",
                module_file.display()
            )
        });
        assert!(
            module_real.starts_with(&venv_real),
            "`{module}` must resolve from this case's environment, not the source tree \
             or `target/`.\n  __file__ = {}\n  .venv    = {}",
            module_real.display(),
            venv_real.display()
        );
        resolved.push(module_real);
    }

    let plot_file = resolved[2].clone();
    let package_dir = plot_file
        .parent()
        .unwrap_or_else(|| panic!("fixture: {} has no parent", plot_file.display()))
        .to_path_buf();
    assert_eq!(
        package_dir.file_name().and_then(|n| n.to_str()),
        Some("mambalibs"),
        "the extension must install into a `mambalibs/` package directory, got {}",
        package_dir.display()
    );
    assert!(
        !package_dir.join("__init__.py").exists(),
        "PEP 420: this wheel must not ship `mambalibs/__init__.py`, so the kit wheels can \
         share the namespace. Found one in {}",
        package_dir.display()
    );
    for (module, file) in modules.iter().zip(&resolved) {
        assert_eq!(
            file.parent(),
            Some(package_dir.as_path()),
            "all three kits must live in one namespace directory, or installing the last \
             has shadowed the earlier ones.\n  {module} = {}\n  mambalibs.plot = {}",
            file.display(),
            plot_file.display()
        );
    }

    // --- step 9: the Python API contract ----------------------------------
    let pytest_dir = pytest_dir();
    assert!(
        pytest_dir.is_dir(),
        "fixture: the pytest directory {} is missing",
        pytest_dir.display()
    );
    let pytest_arg = path_arg(&pytest_dir);
    let pytest_args = [
        "-m",
        "pytest",
        "-p",
        "no:cacheprovider",
        "-q",
        pytest_arg.as_str(),
    ];
    let out = Command::new(&venv_python)
        .args(pytest_args)
        .current_dir(&project)
        .env("HOME", &home)
        .env_remove("PYTHONPATH")
        .env_remove("VIRTUAL_ENV")
        .output()
        .unwrap_or_else(|e| panic!("spawn {} {pytest_args:?}: {e}", venv_python.display()));
    expect_ok("python -m pytest", "python", &pytest_args, &out);

    let report = format!("{}{}", stdout_of(&out), String::from_utf8_lossy(&out.stderr));
    assert!(
        !report.contains("no tests ran"),
        "step `python -m pytest` must collect at least one test from {}: {}",
        pytest_dir.display(),
        render("python", &pytest_args, &out)
    );
    let words: Vec<&str> = report.split_whitespace().collect();
    let passed: usize = words
        .windows(2)
        .find(|w| w[1].starts_with("passed"))
        .and_then(|w| w[0].parse().ok())
        .unwrap_or_else(|| {
            panic!(
                "step `python -m pytest` printed no `<n> passed` summary: {}",
                render("python", &pytest_args, &out)
            )
        });
    assert!(
        passed >= 1,
        "step `python -m pytest` must collect and pass at least one test, saw {passed}: {}",
        render("python", &pytest_args, &out)
    );
}
