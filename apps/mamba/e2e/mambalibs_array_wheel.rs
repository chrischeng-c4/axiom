//! Black-box contract: `mambalibs-arraykit` ships as a PyO3 `abi3` wheel that
//! the mamba package manager can freeze, resolve, install, and import as
//! `mambalibs.array`.
//!
//! # What this case owns
//!
//! The case drives real tools over a throwaway tree and reads only what they
//! wrote. It links no product type: the wheel is built by `maturin` from
//! `apps/mamba/mambalibs/arraykit/pyo3`, frozen by `mamba index build`, locked
//! by `mamba add --index`, installed by `mamba sync`, and finally judged by the
//! project interpreter itself. A green run means those five steps agree on one
//! artifact — a wrong ABI tag, a wheel that lands outside the environment, or
//! an import that resolves from the source tree each fails one named step.
//!
//! # The observation point
//!
//! The environment's interpreter is asked where the module came from
//! (`mambalibs.array.__file__`), and the path is compared against this case's
//! own `.venv`. Nothing is derived from a layout constant, so an
//! implementation that invents its own directory cannot satisfy the
//! assertions by agreeing with the test's arithmetic.
//!
//! # Steps, in order
//!
//! 1. `maturin build` produces exactly one `.whl`, and its tags contain
//!    `cp312-abi3`.
//! 2. `mamba index build` freezes it into a local index directory.
//! 3. `mamba init` scaffolds a PEP 621 `pyproject.toml`; `mamba venv
//!    --system-site-packages` builds the environment on `python3.12`.
//! 4. `mamba add mambalibs-arraykit --index <dir>` locks the index entry.
//! 5. `mamba sync` installs it.
//! 6. `python -c "import mambalibs.array as m; print(m.__file__)"` prints a
//!    path under this case's `.venv`, with no `mambalibs/__init__.py` beside
//!    it (PEP 420).
//! 7. `python -m pytest -p no:cacheprovider` over `e2e/mambalibs_array/`
//!    collects at least one test and exits 0.
//!
//! # Facets
//!
//! - **Behavior**: the seven steps above. The wheel is the only source of
//!   `mambalibs.array`: the case asserts `__file__` is under its own `.venv`,
//!   so an import satisfied by the source tree, by `target/`, or by an
//!   ambient install fails rather than passes.
//! - **Security (fail-closed inputs)**: the environment is stripped of
//!   `MAMBA_FROZEN_INDEX`, `MAMBA_INDEX_URL`, `MAMBA_PYTHON_TAG`,
//!   `VIRTUAL_ENV`, `PYTHONPATH`, and `XDG_CACHE_HOME`, and `HOME` and
//!   `MAMBA_CACHE_DIR` are pinned inside the temp tree, so no ambient value
//!   can supply an answer the case then credits to the product. The only
//!   index is the one the case just built: nothing is fetched from PyPI, and
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
//! with a message naming the missing tool — there is no `#[ignore]`, no
//! early return, and no print-and-pass, because a case that goes green on an
//! unprepared host would report the toolchain instead of the product.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// `CARGO_MANIFEST_DIR` is `apps/mamba`, so the PyO3 crate and the pytest
/// directory are found without knowing where the repository is checked out.
fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn pyo3_crate_dir() -> PathBuf {
    manifest_dir().join("mambalibs").join("arraykit").join("pyo3")
}

fn pytest_dir() -> PathBuf {
    manifest_dir().join("e2e").join("mambalibs_array")
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

/// The host interpreter must be able to `import pytest`: step 7 runs it
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
/// as well: the `cp312-abi3` wheel must be accepted by the shipped selector.
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

/// Every `.whl` directly under `dir`.
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

#[test]
fn arraykit_ships_as_an_abi3_wheel_importable_as_mambalibs_array() {
    let python312 = resolve_python312();
    require_pytest(&python312);
    let maturin = tool_on_path("maturin", "it builds the PyO3 wheel this case installs");

    let tmp = tempfile::tempdir().expect("create temp root");
    let root = tmp.path().to_path_buf();
    let dist = root.join("dist");
    let index = root.join("index");
    let project = root.join("project");
    let home = root.join("home");
    for dir in [&dist, &project, &home] {
        std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("create {}: {e}", dir.display()));
    }

    // --- step 1: maturin build ------------------------------------------
    let crate_dir = pyo3_crate_dir();
    assert!(
        crate_dir.join("Cargo.toml").is_file(),
        "step `maturin build` needs the PyO3 crate at {}, and it does not exist. \
         `mambalibs-arraykit` has no wheel to build until that crate ships: a cdylib \
         bound with pyo3's abi3-py312 feature, and a pyproject.toml whose [tool.maturin] \
         puts the module at `mambalibs.array`.",
        crate_dir.display()
    );

    let dist_arg = path_arg(&dist);
    let interpreter_arg = path_arg(&python312);
    let maturin_args = [
        "build",
        "--out",
        dist_arg.as_str(),
        "--interpreter",
        interpreter_arg.as_str(),
    ];
    let out = Command::new(&maturin)
        .args(maturin_args)
        .current_dir(&crate_dir)
        .env_remove("VIRTUAL_ENV")
        .env_remove("PYTHONPATH")
        .output()
        .unwrap_or_else(|e| panic!("spawn {} {maturin_args:?}: {e}", maturin.display()));
    expect_ok("maturin build", "maturin", &maturin_args, &out);

    let built = wheels_in(&dist);
    assert_eq!(
        built.len(),
        1,
        "step `maturin build` must leave exactly one wheel in {}, found {:?}\n{}",
        dist.display(),
        built
            .iter()
            .map(|p| p.file_name().and_then(|n| n.to_str()).unwrap_or("<?>"))
            .collect::<Vec<_>>(),
        render("maturin", &maturin_args, &out)
    );
    let wheel = built[0].clone();
    let wheel_name = wheel
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_else(|| panic!("fixture: wheel name is not utf-8: {}", wheel.display()))
        .to_string();
    assert!(
        wheel_name.starts_with("mambalibs_arraykit-"),
        "the wheel must carry the distribution name `mambalibs-arraykit`, got {wheel_name}"
    );
    assert!(
        wheel_name.contains("cp312-abi3"),
        "the wheel's tags must contain `cp312-abi3` (the frozen ABI decision), got {wheel_name}"
    );

    // --- step 2: mamba index build --------------------------------------
    let index_arg = path_arg(&index);
    let wheel_arg = path_arg(&wheel);
    let args = [
        "index",
        "build",
        "--out",
        index_arg.as_str(),
        wheel_arg.as_str(),
    ];
    let out = run_mamba(&project, &home, &args);
    expect_ok("mamba index build", "mamba", &args, &out);

    // --- step 3: mamba init + mamba venv --------------------------------
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

    // --- step 4: mamba add ----------------------------------------------
    let args = ["add", "mambalibs-arraykit", "--index", index_arg.as_str()];
    let out = run_mamba(&project, &home, &args);
    expect_ok("mamba add --index", "mamba", &args, &out);

    // --- step 5: mamba sync ---------------------------------------------
    let args = ["sync"];
    let out = run_mamba(&project, &home, &args);
    expect_ok("mamba sync", "mamba", &args, &out);

    // --- step 6: import, from the environment and nowhere else ----------
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

    let import_args = ["-c", "import mambalibs.array as m; print(m.__file__)"];
    let out = Command::new(&venv_python)
        .args(import_args)
        .current_dir(&project)
        .env("HOME", &home)
        .env_remove("PYTHONPATH")
        .env_remove("VIRTUAL_ENV")
        .output()
        .unwrap_or_else(|e| panic!("spawn {} {import_args:?}: {e}", venv_python.display()));
    expect_ok("import mambalibs.array", "python", &import_args, &out);

    let module_file = PathBuf::from(stdout_of(&out).trim());
    let venv_real = std::fs::canonicalize(&venv).unwrap_or_else(|_| venv.clone());
    let module_real = std::fs::canonicalize(&module_file).unwrap_or_else(|e| {
        panic!(
            "`mambalibs.array.__file__` is {} and cannot be canonicalized: {e}",
            module_file.display()
        )
    });
    assert!(
        module_real.starts_with(&venv_real),
        "`mambalibs.array` must resolve from this case's environment, not the source tree \
         or `target/`.\n  __file__ = {}\n  .venv    = {}",
        module_real.display(),
        venv_real.display()
    );

    let package_dir = module_real
        .parent()
        .unwrap_or_else(|| panic!("fixture: {} has no parent", module_real.display()))
        .to_path_buf();
    assert_eq!(
        package_dir.file_name().and_then(|n| n.to_str()),
        Some("mambalibs"),
        "the extension must install into a `mambalibs/` package directory, got {}",
        package_dir.display()
    );
    assert!(
        !package_dir.join("__init__.py").exists(),
        "PEP 420: this wheel must not ship `mambalibs/__init__.py`, so later kit wheels can \
         share the namespace. Found one in {}",
        package_dir.display()
    );

    // --- step 7: pytest --------------------------------------------------
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
