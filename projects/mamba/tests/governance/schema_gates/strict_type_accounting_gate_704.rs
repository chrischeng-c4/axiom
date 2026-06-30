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
fn platform_specific_type_fixtures_are_not_current_platform_oracles() {
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

oracle_tool = pathlib.Path("tests/harness/cpython/tools/verify_cpython_oracle.py")
oracle_spec = importlib.util.spec_from_file_location("verify_cpython_oracle", oracle_tool)
oracle_module = importlib.util.module_from_spec(oracle_spec)
assert oracle_spec.loader is not None
sys.modules[oracle_spec.name] = oracle_module
oracle_spec.loader.exec_module(oracle_module)

strict_winapi_fixture = strict_module.TYPE_DIR / "std-libs/_winapi/WaitForMultipleObjects__handle_seq_as_Sequence_wrong.py"
oracle_winapi_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/_winapi/WaitForMultipleObjects__handle_seq_as_Sequence_wrong.py"
tkinter_fixture = strict_module.TYPE_DIR / "std-libs/_tkinter/TkappType__wantobjects__wantobjects_as_typed_wrong.py"

expected = sys.platform != "win32"
assert strict_module.PLATFORM_SPECIFIC_TYPE_LIBS["_winapi"] == "win32"
assert oracle_module.PLATFORM_SPECIFIC_TYPE_LIBS["_winapi"] == "win32"
assert strict_module.is_platform_specific_unavailable_type_fixture(strict_winapi_fixture) == expected
assert oracle_module.is_platform_specific_unavailable_type_fixture(oracle_winapi_fixture) == expected
assert not strict_module.is_platform_specific_unavailable_type_fixture(tkinter_fixture)
if expected:
    assert strict_winapi_fixture not in strict_module.executable_type_fixtures([strict_winapi_fixture])
"#;
    let output = Command::new("python3.12")
        .arg("-c")
        .arg(script)
        .current_dir(mamba_root())
        .output()
        .expect("run platform-specific type fixture smoke");
    assert!(
        output.status.success(),
        "platform-specific type fixture smoke failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn version_specific_type_fixtures_are_not_py312_oracles() {
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

oracle_tool = pathlib.Path("tests/harness/cpython/tools/verify_cpython_oracle.py")
oracle_spec = importlib.util.spec_from_file_location("verify_cpython_oracle", oracle_tool)
oracle_module = importlib.util.module_from_spec(oracle_spec)
assert oracle_spec.loader is not None
sys.modules[oracle_spec.name] = oracle_module
oracle_spec.loader.exec_module(oracle_module)

strict_zstd_fixture = strict_module.TYPE_DIR / "std-libs/_zstd/finalize_dict__custom_dict_bytes_as_bytes_wrong.py"
strict_compression_fixture = strict_module.TYPE_DIR / "std-libs/compression_zstd/compress__data_as_ReadableBuffer_wrong.py"
strict_annotationlib_fixture = strict_module.TYPE_DIR / "std-libs/annotationlib/ForwardRef__init__arg_as_str_wrong.py"
strict_asyncio_graph_fixture = strict_module.TYPE_DIR / "std-libs/asyncio_graph/capture_call_graph__future_as_Future_wrong.py"
strict_asyncio_tools_fixture = strict_module.TYPE_DIR / "std-libs/asyncio_tools/CycleFoundException__init__cycles_as_list_wrong.py"
strict_templatestr_fixture = strict_module.TYPE_DIR / "std-libs/ast/TemplateStr__init__values_as_list_wrong.py"
strict_asynchat_fixture = strict_module.TYPE_DIR / "std-libs/asynchat/async_chat__push__data_as_bytes_wrong.py"
strict_asyncio_coroutine_fixture = strict_module.TYPE_DIR / "std-libs/asyncio_coroutines/coroutine__func_as__FunctionT_wrong.py"
oracle_zstd_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/_zstd/finalize_dict__custom_dict_bytes_as_bytes_wrong.py"
oracle_annotationlib_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/annotationlib/ForwardRef__init__arg_as_str_wrong.py"
oracle_asyncio_graph_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/asyncio_graph/capture_call_graph__future_as_Future_wrong.py"
oracle_asyncio_tools_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/asyncio_tools/CycleFoundException__init__cycles_as_list_wrong.py"
oracle_templatestr_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/ast/TemplateStr__init__values_as_list_wrong.py"
oracle_asynchat_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/asynchat/async_chat__push__data_as_bytes_wrong.py"
oracle_asyncio_coroutine_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/asyncio_coroutines/coroutine__func_as__FunctionT_wrong.py"
tkinter_fixture = strict_module.TYPE_DIR / "std-libs/_tkinter/TkappType__wantobjects__wantobjects_as_typed_wrong.py"

expected = sys.version_info[:2] < (3, 14)
expected_removed = sys.version_info[:2] >= (3, 12)
assert strict_module.VERSION_SPECIFIC_TYPE_LIBS["annotationlib"] == (3, 14)
assert strict_module.VERSION_SPECIFIC_TYPE_LIBS["_zstd"] == (3, 14)
assert strict_module.VERSION_SPECIFIC_TYPE_LIBS["asyncio_graph"] == (3, 14)
assert strict_module.VERSION_SPECIFIC_TYPE_LIBS["asyncio_tools"] == (3, 14)
assert strict_module.VERSION_SPECIFIC_TYPE_LIBS["compression_zstd"] == (3, 14)
assert strict_module.VERSION_SPECIFIC_TYPE_LIBS["compression_zstd__zstdfile"] == (3, 14)
assert strict_module.VERSION_REMOVED_TYPE_LIBS["asynchat"] == (3, 12)
assert strict_module.VERSION_REMOVED_TYPE_LIBS["asyncore"] == (3, 12)
assert strict_module.VERSION_REMOVED_TYPE_LIBS["smtpd"] == (3, 12)
assert strict_module.VERSION_SPECIFIC_TYPE_FIXTURES["std-libs/ast/TemplateStr__init__values_as_list_wrong.py"] == (3, 14)
assert strict_module.VERSION_REMOVED_TYPE_FIXTURES["std-libs/asyncio_coroutines/coroutine__func_as__FunctionT_wrong.py"] == (3, 12)
assert oracle_module.VERSION_SPECIFIC_TYPE_LIBS["annotationlib"] == (3, 14)
assert oracle_module.VERSION_SPECIFIC_TYPE_LIBS["_zstd"] == (3, 14)
assert oracle_module.VERSION_SPECIFIC_TYPE_LIBS["asyncio_graph"] == (3, 14)
assert oracle_module.VERSION_SPECIFIC_TYPE_LIBS["asyncio_tools"] == (3, 14)
assert oracle_module.VERSION_REMOVED_TYPE_LIBS["asynchat"] == (3, 12)
assert oracle_module.VERSION_REMOVED_TYPE_LIBS["asyncore"] == (3, 12)
assert oracle_module.VERSION_REMOVED_TYPE_LIBS["smtpd"] == (3, 12)
assert oracle_module.VERSION_SPECIFIC_TYPE_FIXTURES["std-libs/ast/TemplateStr__init__values_as_list_wrong.py"] == (3, 14)
assert oracle_module.VERSION_REMOVED_TYPE_FIXTURES["std-libs/asyncio_coroutines/coroutine__func_as__FunctionT_wrong.py"] == (3, 12)
assert strict_module.is_version_specific_unavailable_type_fixture(strict_annotationlib_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_zstd_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_compression_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_asyncio_graph_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_asyncio_tools_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_templatestr_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_asynchat_fixture) == expected_removed
assert strict_module.is_version_specific_unavailable_type_fixture(strict_asyncio_coroutine_fixture) == expected_removed
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_annotationlib_fixture) == expected
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_zstd_fixture) == expected
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_asyncio_graph_fixture) == expected
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_asyncio_tools_fixture) == expected
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_templatestr_fixture) == expected
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_asynchat_fixture) == expected_removed
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_asyncio_coroutine_fixture) == expected_removed
assert not strict_module.is_version_specific_unavailable_type_fixture(tkinter_fixture)
if expected:
    assert strict_annotationlib_fixture not in strict_module.executable_type_fixtures([strict_annotationlib_fixture])
    assert strict_zstd_fixture not in strict_module.executable_type_fixtures([strict_zstd_fixture])
    assert strict_asyncio_graph_fixture not in strict_module.executable_type_fixtures([strict_asyncio_graph_fixture])
    assert strict_asyncio_tools_fixture not in strict_module.executable_type_fixtures([strict_asyncio_tools_fixture])
    assert strict_templatestr_fixture not in strict_module.executable_type_fixtures([strict_templatestr_fixture])
if expected_removed:
    assert strict_asynchat_fixture not in strict_module.executable_type_fixtures([strict_asynchat_fixture])
    assert strict_asyncio_coroutine_fixture not in strict_module.executable_type_fixtures([strict_asyncio_coroutine_fixture])
"#;
    let output = Command::new("python3.12")
        .arg("-c")
        .arg(script)
        .current_dir(mamba_root())
        .output()
        .expect("run version-specific type fixture smoke");
    assert!(
        output.status.success(),
        "version-specific type fixture smoke failed\nstdout={}\nstderr={}",
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
fn abc_unknown_contracts_are_not_strict_type_walls() {
    let root = mamba_root();
    for path in [
        "tests/cpython/type/std-libs/abc/ABCMeta____subclasscheck____subclass_as_type_wrong.py",
        "tests/cpython/type/std-libs/abc/ABCMeta__register__subclass_as_type_wrong.py",
        "tests/cpython/type/std-libs/abc/abstractclassmethod__init__callable_as_Callable_wrong.py",
        "tests/cpython/type/std-libs/abc/abstractmethod__funcobj_as__FuncT_wrong.py",
        "tests/cpython/type/std-libs/abc/abstractstaticmethod__init__callable_as_Callable_wrong.py",
        "tests/cpython/type/std-libs/abc/update_abstractmethods__cls_as_type_wrong.py",
    ] {
        assert!(
            !root.join(path).exists(),
            "abc Unknown/Callable/TypeVar params must not be executable strict walls: {path}"
        );
    }
    assert!(
        root.join("tests/cpython/type/std-libs/abc/ABCMeta____new____name_as_str_wrong.py")
            .exists(),
        "abc.ABCMeta.__new__(name: str) must remain as the enforceable abc strict wall"
    );
    assert!(
        root.join("tests/cpython/surface/std-libs/abc/abcmeta_has_register.py")
            .exists(),
        "abc register behavior still needs executable surface coverage"
    );
}

#[test]
fn aifc_open_unknown_contract_is_not_a_strict_type_wall() {
    let root = mamba_root();
    assert!(
        !root
            .join("tests/cpython/type/std-libs/aifc/open__f_as__File_wrong.py")
            .exists(),
        "aifc.open(f: _File) emits Unknown/non-enforceable and must not be a strict wall"
    );
    assert!(
        root.join("tests/cpython/type/std-libs/aifc/Aifc_read__getmark__id_as_int_wrong.py")
            .exists(),
        "aifc scalar method walls must remain enforced while module open(f) is skipped"
    );
}

#[test]
fn argparse_unknown_contracts_are_not_strict_type_walls() {
    let root = mamba_root();
    for rel in [
        "Action__init__option_strings_as_Sequence_wrong.py",
        "ArgumentParser__format_help__formatter_as_typed_wrong.py",
        "ArgumentParser__format_usage__formatter_as_typed_wrong.py",
        "ArgumentParser__parse_args__args_as_typed_wrong.py",
        "ArgumentParser__parse_intermixed_args__args_as_typed_wrong.py",
        "ArgumentParser__parse_known_args__args_as_typed_wrong.py",
        "ArgumentParser__parse_known_intermixed_args__args_as_typed_wrong.py",
        "BooleanOptionalAction__init__option_strings_as_Sequence_wrong.py",
    ] {
        assert!(
            !root
                .join("tests/cpython/type/std-libs/argparse")
                .join(rel)
                .exists(),
            "argparse Unknown/non-enforceable param must not be an executable strict wall: {rel}"
        );
    }
    for rel in [
        "ArgumentParser__error__message_as_str_wrong.py",
        "ArgumentParser__exit__status_as_int_wrong.py",
        "FileType__init__mode_as_str_wrong.py",
    ] {
        assert!(
            root.join("tests/cpython/type/std-libs/argparse")
                .join(rel)
                .exists(),
            "argparse scalar strict wall must remain enforced: {rel}"
        );
    }
}

#[test]
fn array_unknown_contracts_are_not_strict_type_walls() {
    let root = mamba_root();
    for rel in [
        "array____add____value_as_array_wrong.py",
        "array____delitem____key_as_typed_wrong.py",
        "array____ge____value_as_array_wrong.py",
        "array____getitem____key_as_SupportsIndex_wrong.py",
        "array____getitem____key_as_slice_wrong.py",
        "array____gt____value_as_array_wrong.py",
        "array____iadd____value_as_array_wrong.py",
        "array____le____value_as_array_wrong.py",
        "array____lt____value_as_array_wrong.py",
        "array____new____typecode_as_Literal_wrong.py",
        "array____new____typecode_as__FloatTypeCode_wrong.py",
        "array____new____typecode_as__IntTypeCode_wrong.py",
        "array____new____typecode_as_str_wrong.py",
        "array____setitem____key_as_SupportsIndex_wrong.py",
        "array____setitem____key_as_slice_wrong.py",
        "array__append__v_as__T_wrong.py",
        "array__count__v_as__T_wrong.py",
        "array__fromlist__list_as_list_wrong.py",
        "array__index__v_as__T_wrong.py",
        "array__remove__v_as__T_wrong.py",
    ] {
        assert!(
            !root
                .join("tests/cpython/type/std-libs/array")
                .join(rel)
                .exists(),
            "array Unknown/non-enforceable param must not be an executable strict wall: {rel}"
        );
    }
    for rel in [
        "array____buffer____flags_as_int_wrong.py",
        "array____imul____value_as_int_wrong.py",
        "array____mul____value_as_int_wrong.py",
        "array____rmul____value_as_int_wrong.py",
        "array__fromunicode__ustr_as_str_wrong.py",
        "array__insert__i_as_int_wrong.py",
        "array__pop__i_as_int_wrong.py",
    ] {
        assert!(
            root.join("tests/cpython/type/std-libs/array")
                .join(rel)
                .exists(),
            "array scalar strict wall must remain enforced: {rel}"
        );
    }
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
assert gen_module.is_not_wrongable(ast.Name(id="type"))
assert gen_module.is_not_wrongable(ast.Name(id="Callable"))
assert not gen_module.is_not_wrongable(ast.Name(id="str"))
assert gen_module.is_signature_param_not_wrongable("aifc", None, "open", "f")
assert gen_module.is_signature_param_not_wrongable("argparse", "Action", "__init__", "option_strings")
assert gen_module.is_signature_param_not_wrongable("argparse", "ArgumentParser", "parse_args", "args")
assert gen_module.is_signature_param_not_wrongable("argparse", "BooleanOptionalAction", "__init__", "option_strings")
assert gen_module.is_signature_param_not_wrongable("array", "array", "__add__", "value")
assert gen_module.is_signature_param_not_wrongable("array", "array", "__getitem__", "key")
assert gen_module.is_signature_param_not_wrongable("array", "array", "__new__", "typecode")
assert gen_module.is_signature_param_not_wrongable("array", "array", "append", "v")
assert not gen_module.is_signature_param_not_wrongable("aifc", "Aifc_read", "getmark", "id")
assert not gen_module.is_signature_param_not_wrongable("argparse", "ArgumentParser", "error", "message")
assert not gen_module.is_signature_param_not_wrongable("array", "array", "__mul__", "value")
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
