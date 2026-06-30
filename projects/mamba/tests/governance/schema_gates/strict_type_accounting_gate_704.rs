//! Executable metadata gate for #704: strict-type accounting must stay
//! machine-readable and wired into replacement readiness.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn project_root() -> PathBuf {
    crate::common::project_root()
}

fn mamba_root() -> PathBuf {
    project_root()
}

fn py_compile(paths: &[PathBuf]) {
    let output = Command::new("python3.12")
        .arg("-m")
        .arg("py_compile")
        .args(paths)
        .current_dir(mamba_root())
        .output()
        .expect("run py_compile");
    assert!(
        output.status.success(),
        "py_compile failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn strict_type_tools_are_python_parseable() {
    let root = mamba_root();
    py_compile(&[
        root.join("tests/harness/cpython/tools/strict_type_accounting.py"),
        root.join("tests/harness/cpython/tools/replacement_readiness.py"),
        root.join("tests/harness/cpython/tools/fixture_lint.py"),
        root.join("tests/harness/cpython/tools/type_wall_gen.py"),
        root.join("tests/harness/cpython/tools/type_enforce_matrix.py"),
        root.join("tests/harness/cpython/tools/verify_cpython_oracle.py"),
    ]);
}

#[test]
fn fixture_lint_supports_type_facet_filter() {
    let output = Command::new("python3.12")
        .arg("tests/harness/cpython/tools/fixture_lint.py")
        .args(["--bucket", "type", "--show", "1"])
        .current_dir(mamba_root())
        .output()
        .expect("run fixture_lint type facet");
    assert!(
        output.status.success(),
        "fixture_lint --bucket type failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("recorded="),
        "fixture_lint output should include fixture counts: {stdout}"
    );
}

#[test]
fn strict_type_accounting_accepts_compile_time_type_error_marker() {
    let script = r#"
import importlib.util
import pathlib
import sys

tool = pathlib.Path("tests/harness/cpython/tools/strict_type_accounting.py")
sys.path.insert(0, str(tool.parent))
spec = importlib.util.spec_from_file_location("strict_type_accounting", tool)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = module
spec.loader.exec_module(module)
assert module.is_type_rejection("", "error: type error at 1..2: rejected")
assert not module.is_type_rejection("", "error: undefined name at 1..2: missing")
"#;
    let output = Command::new("python3.12")
        .arg("-c")
        .arg(script)
        .current_dir(mamba_root())
        .output()
        .expect("run strict_type_accounting marker smoke");
    assert!(
        output.status.success(),
        "strict_type_accounting marker smoke failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn replacement_readiness_uses_strict_type_accounting_tool() {
    let text = fs::read_to_string(
        mamba_root().join("tests/harness/cpython/tools/replacement_readiness.py"),
    )
    .expect("read replacement_readiness.py");
    assert!(text.contains("STRICT_TYPE_ACCOUNTING"));
    assert!(text.contains("strict_type_dimension"));
    assert!(text.contains("type_enforced"));
    assert!(
        !text.contains(
            "strict-type denominator and verified divergence accounting are not yet integrated"
        ),
        "strict-type readiness must not regress to a blocked placeholder"
    );
}

#[test]
fn non_runtime_typeshed_stubs_are_not_executable_type_fixtures() {
    let script = r#"
import importlib.util
import pathlib
import sys

strict_tool = pathlib.Path("tests/harness/cpython/tools/strict_type_accounting.py")
sys.path.insert(0, str(strict_tool.parent))
strict_spec = importlib.util.spec_from_file_location("strict_type_accounting", strict_tool)
strict_module = importlib.util.module_from_spec(strict_spec)
assert strict_spec.loader is not None
sys.modules[strict_spec.name] = strict_module
strict_spec.loader.exec_module(strict_module)

typeshed_fixture = strict_module.TYPE_DIR / "std-libs/_typeshed/IdentityFunction____call____x_as__T_wrong.py"
typeshed_internal_fixture = strict_module.TYPE_DIR / "std-libs/_typeshed__type_checker_internals/TypedDictFallback__pop__k_as_Never_wrong.py"
typeshed_dbapi_fixture = strict_module.TYPE_DIR / "std-libs/_typeshed_dbapi/DBAPICursor__fetchmany__size_as_int_wrong.py"
tkinter_fixture = strict_module.TYPE_DIR / "std-libs/_tkinter/TkappType__wantobjects__wantobjects_as_typed_wrong.py"
assert strict_module.is_non_runtime_stub_type_fixture(typeshed_fixture)
assert strict_module.is_non_runtime_stub_type_fixture(typeshed_internal_fixture)
assert strict_module.is_non_runtime_stub_type_fixture(typeshed_dbapi_fixture)
assert not strict_module.is_non_runtime_stub_type_fixture(tkinter_fixture)

gen_tool = pathlib.Path("tests/harness/cpython/tools/type_wall_gen.py")
gen_spec = importlib.util.spec_from_file_location("type_wall_gen", gen_tool)
gen_module = importlib.util.module_from_spec(gen_spec)
assert gen_spec.loader is not None
sys.modules[gen_spec.name] = gen_module
gen_spec.loader.exec_module(gen_module)
assert "_typeshed" in gen_module.NON_RUNTIME_STUB_MODULE_PREFIXES
"#;
    let output = Command::new("python3.12")
        .arg("-c")
        .arg(script)
        .current_dir(mamba_root())
        .output()
        .expect("run non-runtime stub fixture smoke");
    assert!(
        output.status.success(),
        "non-runtime stub fixture smoke failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn warnings_strict_type_wall_is_curated() {
    let text =
        fs::read_to_string(mamba_root().join("src/types/stdlib_sigs.rs")).expect("read sig table");
    for name in ["warn", "warn_explicit"] {
        let needle =
            format!("module: \"_warnings\",\n        qualifier: \"\",\n        name: \"{name}\"");
        let row_start = text
            .find(&needle)
            .unwrap_or_else(|| panic!("missing curated _warnings.{name} row"));
        let rest = &text[row_start..];
        let row_end = rest.find("\n    StdlibSig {").unwrap_or(rest.len());
        let row = &rest[..row_end];
        assert!(
            row.contains("p(\"message\", CoreTy::Str)"),
            "_warnings.{name} must keep a strict scalar wall for message"
        );
    }
}

#[test]
fn weakrefset_constructor_strict_type_wall_is_curated() {
    let text =
        fs::read_to_string(mamba_root().join("src/types/stdlib_sigs.rs")).expect("read sig table");
    let needle =
        "module: \"_weakrefset\",\n        qualifier: \"WeakSet\",\n        name: \"__init__\"";
    let row_start = text
        .find(needle)
        .expect("missing curated _weakrefset.WeakSet.__init__ row");
    let rest = &text[row_start..];
    let row_end = rest.find("\n    StdlibSig {").unwrap_or(rest.len());
    let row = &rest[..row_end];
    assert!(
        row.contains("p(\"data\", CoreTy::Typed)"),
        "_weakrefset.WeakSet.__init__ must keep a strict Typed wall for data"
    );
}

#[test]
fn weakref_proxy_typevars_are_surface_not_strict_walls() {
    let root = mamba_root();
    for path in [
        "tests/cpython/type/std-libs/_weakref/proxy__object_as__C_wrong.py",
        "tests/cpython/type/std-libs/_weakref/proxy__object_as__T_wrong.py",
    ] {
        assert!(
            !root.join(path).exists(),
            "_weakref.proxy TypeVar params are Unknown and must not be executable strict walls: {path}"
        );
    }
    assert!(
        root.join("tests/cpython/surface/std-libs/_weakref/proxy_accepts_user_object.py")
            .exists(),
        "_weakref.proxy still needs executable surface coverage"
    );
}

#[test]
fn type_wall_generator_skips_typevar_fixture_params() {
    let script = r#"
import ast
import importlib.util
import pathlib
import sys

gen_tool = pathlib.Path("tests/harness/cpython/tools/type_wall_gen.py")
sys.path.insert(0, str(gen_tool.parent))
gen_spec = importlib.util.spec_from_file_location("type_wall_gen", gen_tool)
gen_module = importlib.util.module_from_spec(gen_spec)
assert gen_spec.loader is not None
sys.modules[gen_spec.name] = gen_module
gen_spec.loader.exec_module(gen_module)
assert gen_module.is_not_wrongable(ast.Name(id="_T"))
assert gen_module.is_not_wrongable(ast.Name(id="_C"))
assert not gen_module.is_not_wrongable(ast.Name(id="str"))
"#;
    let output = Command::new("python3.12")
        .arg("-c")
        .arg(script)
        .current_dir(mamba_root())
        .output()
        .expect("run type-wall TypeVar skip smoke");
    assert!(
        output.status.success(),
        "type-wall TypeVar skip smoke failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn declared_type_divergences_have_machine_owner_refs() {
    let path = mamba_root().join("tests/harness/cpython/config/type_divergences.txt");
    let text = fs::read_to_string(path).expect("read type_divergences.txt");
    let mut current_owner = false;
    let mut entries = 0usize;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            if line.contains("owner:") && line.contains('#') {
                current_owner = true;
            }
            continue;
        }
        entries += 1;
        assert!(
            current_owner,
            "type divergence entry lacks preceding '# owner: #<issue>' line: {line}"
        );
        assert!(
            line.starts_with("projects/mamba/tests/cpython/"),
            "type divergence must use repo-relative fixture path: {line}"
        );
        current_owner = false;
    }
    assert!(
        entries > 0,
        "expected at least one declared type divergence"
    );
}

#[test]
fn generated_typeshed_denominator_header_is_present() {
    let text = fs::read_to_string(mamba_root().join("src/types/stdlib_sigs_generated.rs"))
        .expect("read generated stdlib sig table");
    assert!(text.contains("rows:"));
    assert!(text.contains("enforceable (scalar):"));
    assert!(text.contains("unknown-skipped:"));
}
