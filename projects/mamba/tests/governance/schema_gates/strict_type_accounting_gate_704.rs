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

fn run_python_script(script: &str) -> std::process::Output {
    Command::new("python3.12")
        .arg("-c")
        .arg(script)
        .current_dir(mamba_root())
        .output()
        .expect("run python script")
}

fn assert_python_script(script: &str, label: &str) {
    let output = Command::new("python3.12")
        .arg("-c")
        .arg(script)
        .current_dir(mamba_root())
        .output()
        .unwrap_or_else(|error| panic!("run {label}: {error}"));
    assert!(
        output.status.success(),
        "{label} failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn mamba_bin() -> PathBuf {
    let project_local = mamba_root().join("target/debug/mamba");
    if project_local.exists() {
        return project_local;
    }
    mamba_root().join("../../target/debug/mamba")
}

fn run_mamba_fixture(fixture_rel: &str) -> std::process::Output {
    Command::new(mamba_bin())
        .arg("run")
        .arg(fixture_rel)
        .current_dir(mamba_root())
        .output()
        .expect("run mamba fixture")
}

#[test]
fn strict_type_tools_are_python_parseable() {
    let root = mamba_root();
    py_compile(&[
        root.join("tests/harness/cpython/tools/strict_type_accounting.py"),
        root.join("tests/harness/cpython/tools/replacement_readiness.py"),
        root.join("tests/harness/cpython/tools/fixture_lint.py"),
        root.join("tests/harness/cpython/tools/checkout_typeshed.py"),
        root.join("tests/harness/cpython/tools/typeshed_lock.py"),
        root.join("tests/harness/cpython/tools/type_wall_gen.py"),
        root.join("tests/harness/cpython/tools/wall_status.py"),
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
fn strict_type_accounting_requires_authoritative_contract_inventory() {
    let script = r#"
import importlib.util
import pathlib
import sys
import tempfile

tool = pathlib.Path("tests/harness/cpython/tools/strict_type_accounting.py")
sys.path.insert(0, str(tool.parent))
spec = importlib.util.spec_from_file_location("strict_type_accounting", tool)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = module
spec.loader.exec_module(module)

assert module.EXPECTED_PYTHON_VERSION == (3, 12)
missing = module.verify_generated_signature_snapshot(
    pathlib.Path("/definitely/missing/typeshed/stdlib")
)
assert not missing["current"]

with tempfile.TemporaryDirectory() as tmp:
    fixture = pathlib.Path(tmp) / "contract.py"
    fixture.write_text('# subject = "demo.f(value: typed)"\n', encoding="utf-8")
    sigs = {
        ("demo", "", "f"): {
            "params": {"value": "unsupported"},
            "enforceable": False,
        }
    }
    excluded, unresolved = module.partition_generated_contract_coverage(
        [fixture], sigs
    )
    assert not excluded
    assert unresolved[0]["reason"] == "structured_param_unsupported"

    fixture.write_text(
        '# subject = "demo.f(value: T)"\n'
        '# TypeVar param must stay unwalled\n',
        encoding="utf-8",
    )
    excluded, unresolved = module.partition_generated_contract_coverage(
        [fixture], sigs
    )
    assert not excluded
    assert unresolved[0]["reason"] == (
        "stale_typevar_unwalled_marker_structured_param_unsupported"
    )

    sigs[("demo", "", "f")]["params"]["value"] = "unconstrained"
    excluded, unresolved = module.partition_generated_contract_coverage(
        [fixture], sigs
    )
    assert excluded[0]["reason"] == "contract_unconstrained"
    assert not unresolved

source = tool.read_text(encoding="utf-8")
assert "host_python_version == EXPECTED_PYTHON_VERSION" in source
assert 'and generated_snapshot["current"]' in source
assert "and not unresolved_generated_contracts" in source
"#;
    let output = Command::new("python3.12")
        .arg("-c")
        .arg(script)
        .current_dir(mamba_root())
        .output()
        .expect("run strict type authoritative inventory smoke");
    assert!(
        output.status.success(),
        "strict type authoritative inventory smoke failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn generated_inventory_tracks_properties_exports_and_py312_branches() {
    let script = r#"
import copy
import importlib.util
import pathlib
import subprocess
import sys

tool = pathlib.Path("tests/harness/cpython/tools/strict_type_accounting.py")
sys.path.insert(0, str(tool.parent))
spec = importlib.util.spec_from_file_location("strict_type_accounting", tool)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = module
spec.loader.exec_module(module)

row = module.parse_generated_signature_param_index()[
    ("urllib.request", "Request", "full_url")
]
assert row["manifest_branches"] == 2
assert row["branches"] == 1
assert row["params"]["value"] == "supported"

exception_index = module.parse_generated_signature_param_index()
for key, param in [
    (("_warnings", "", "warn"), "message"),
    (("codecs", "", "strict_errors"), "exception"),
    (("asyncio.streams", "StreamReader", "set_exception"), "exc"),
    (("traceback", "", "print_exception"), "exc"),
    (("unittest.case", "TestCase", "assertRaises"), "expected_exception"),
    (("asyncio.subprocess", "SubprocessStreamProtocol", "__init__"), "loop"),
    (("select", "", "select"), "rlist"),
]:
    assert exception_index[key]["params"][param] == "supported", (key, param)

generic_index = module.parse_generated_signature_param_index()
xml_init = generic_index[("xml.etree.ElementTree", "XMLPullParser", "__init__")]
assert xml_init["params"]["events"] == "supported"
buffered_reader = generic_index[("_io", "BufferedReader", "__init__")]
assert buffered_reader["params"]["buffer_size"] == "supported"
assert buffered_reader["params"]["raw"] == "unsupported"
assert buffered_reader["param_reasons"]["raw"] == "structured_param_type_unsupported"
assert all(
    reason != "structured_generic_metadata_unsupported"
    for row in generic_index.values()
    for reason in row["param_reasons"].values()
)

for key, param in [
    (("_contextvars", "Context", "run"), "callable"),
    (("curses", "", "wrapper"), "func"),
]:
    row = generic_index[key]
    assert row["params"][param] == "supported", (key, param, row)
    assert param not in row["param_reasons"]

for key, params in [
    (("_contextvars", "Context", "run"), {"args", "kwargs"}),
    (("curses", "", "wrapper"), {"arg", "kwds"}),
]:
    row = generic_index[key]
    assert all(row["params"][param] == "supported" for param in params), row

for key in [
    ("builtins", "staticmethod", "__call__"),
    ("functools", "_Wrapped", "__call__"),
]:
    row = generic_index[key]
    for param in ("args", "kwargs"):
        assert row["params"][param] == "unsupported", (key, param, row)
        assert row["param_reasons"][param] == "structured_param_type_unsupported"

manifest = module.load_generated_typespec_manifest()
strings = manifest["strings"]
def param_node(key, name):
    row = next(
        row for row in module._generated_callable_rows(
            manifest, include_class_inventory=True
        )
        if tuple(strings[row[index]] for index in range(3)) == key
    )
    start, length = row[4]
    param = next(
        param for param in manifest["params"][start : start + length]
        if strings[param[0]] == name
    )
    return manifest["type_uses"][param[2]][0]

context_args = param_node(("_contextvars", "Context", "run"), "args")
context_callable = param_node(("_contextvars", "Context", "run"), "callable")
assert module._generated_typespec_status(manifest, context_args) == "unsupported"
assert module._callable_paramspec_shape(manifest, context_callable) is not None

curses_callable = param_node(("curses", "", "wrapper"), "func")
callable_value = manifest["nodes"][curses_callable]["Apply"]
callable_start, _ = callable_value["args"]
concat_node, return_node = manifest["edges"][callable_start : callable_start + 2]
concat_value = manifest["nodes"][concat_node]["Apply"]
concat_start, concat_length = concat_value["args"]
param_spec_node = manifest["edges"][concat_start + concat_length - 1]

def malformed_callable(concat_args):
    malformed = copy.deepcopy(manifest)
    concat_start = len(malformed["edges"])
    malformed["edges"].extend(concat_args)
    bad_concat = len(malformed["nodes"])
    malformed["nodes"].append({
        "Apply": {"base": concat_value["base"], "args": [concat_start, len(concat_args)]}
    })
    callable_start = len(malformed["edges"])
    malformed["edges"].extend([bad_concat, return_node])
    bad_callable = len(malformed["nodes"])
    malformed["nodes"].append({
        "Apply": {"base": callable_value["base"], "args": [callable_start, 2]}
    })
    return malformed, bad_callable

for concat_args in ([param_spec_node], [param_spec_node, param_spec_node]):
    malformed, bad_callable = malformed_callable(concat_args)
    assert module._callable_paramspec_shape(malformed, bad_callable) is None
    assert module._generated_typespec_status(malformed, bad_callable) == "unsupported"

property_fixture = module.TYPE_DIR / "std-libs/urllib_request/Request__full_url__value_as_str_wrong.py"
property_text = property_fixture.read_text(encoding="utf-8")
assert '# subject = "urllib.request.Request.full_url = (value: str)"' in property_text
assert "obj: Request = Request.__new__(Request)" in property_text
assert "obj.full_url = 12345" in property_text
assert property_fixture in module.executable_type_fixtures([property_fixture])
assert module.unenforceable_generated_param_reason(
    property_fixture, module.parse_generated_signature_param_index()
) is None

object_property = module.TYPE_DIR / "builtin-libs/builtins/object____class____type_as_type_wrong.py"
object_text = object_property.read_text(encoding="utf-8")
assert "from builtins import object as Object" in object_text
assert "obj: Object = Object()" in object_text
abstract_property = module.TYPE_DIR / "std-libs/_ctypes/Array__raw__value_as_ReadableBuffer_wrong.py"
assert not abstract_property.exists()

inactive_fixture = module.TYPE_DIR / "std-libs/pathlib/Path__copy__target_as_StrPath_wrong.py"
assert module.is_generated_typespec_inactive_type_fixture(inactive_fixture)
assert inactive_fixture not in module.executable_type_fixtures([inactive_fixture])

private_fixture = module.TYPE_DIR / "std-libs/_heapq/_heapify_max__heap_as_list_wrong.py"
assert not private_fixture.exists()

expanded = module.parse_generated_signature_param_index()
canonical = module.parse_generated_signature_param_index(expand_exports=False)
assert ("_heapq", "", "_heapify_max") not in expanded
assert ("ntpath", "", "commonpath") in expanded
assert ("ntpath", "", "commonpath") not in canonical
assert ("asyncio", "AbstractEventLoopPolicy", "get_event_loop") in expanded
assert ("asyncio", "AbstractEventLoopPolicy", "get_event_loop") not in canonical
assert ("threading", "local", "__getattribute__") in expanded
assert ("threading", "local", "__getattribute__") not in canonical
counts = module.parse_generated_signature_counts(pathlib.Path("."))
assert counts["rows"] == len(canonical)
assert len(expanded) > len(canonical)
assert counts["unhandled_binding_branches"] == 0

synthetic = {("example", "C", "value"): {"g": True, "t": False}}
assert module._generated_contract_is_inactive(
    "example.C.value", "property_set", synthetic
)
assert module._generated_contract_is_inactive(
    "example.C.value", "call", synthetic
)
assert not module._generated_contract_is_inactive(
    "example.C.value", "property_set", {("example", "C", "value"): {"g": True}}
)

sqlite_property = module.TYPE_DIR / "std-libs/sqlite3/Connection__autocommit__val_as_int_wrong.py"
sqlite_text = sqlite_property.read_text(encoding="utf-8")
assert 'obj: Connection = Connection(":memory:")' in sqlite_text
sqlite_oracle = subprocess.run(
    ["python3.12", str(sqlite_property)], text=True, capture_output=True, check=False
)
assert sqlite_oracle.returncode == 0
assert "setup_or_other:" not in sqlite_oracle.stdout
assert sqlite_oracle.stdout.startswith(("typeerror:", "no_typeerror:", "assignment_other:"))

oracle_tool = pathlib.Path("tests/harness/cpython/tools/verify_cpython_oracle.py")
oracle_spec = importlib.util.spec_from_file_location("verify_cpython_oracle", oracle_tool)
oracle_module = importlib.util.module_from_spec(oracle_spec)
assert oracle_spec.loader is not None
sys.modules[oracle_spec.name] = oracle_module
oracle_spec.loader.exec_module(oracle_module)
assert oracle_module.is_generated_typespec_inactive_type_fixture(inactive_fixture)
oracle_result = oracle_module.run_one(
    inactive_fixture, "python3.12", 1.0, False, set()
)
assert oracle_result.status == "skip"
assert oracle_result.reason == "inactive-typespec-contract"
"#;
    let output = Command::new("python3.12")
        .arg("-c")
        .arg(script)
        .current_dir(mamba_root())
        .output()
        .expect("run generated inventory accounting smoke");
    assert!(
        output.status.success(),
        "generated inventory accounting smoke failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn protocol_accounting_requires_complete_enforceable_members() {
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

manifest = module.load_generated_typespec_manifest()
assert module._generated_protocol_status(manifest, "typing", "SupportsIndex") == "supported"
assert module._generated_protocol_status(manifest, "collections.abc", "Iterable") == "supported"
assert module._generated_protocol_status(manifest, "os", "PathLike") == "supported"
assert module._generated_protocol_status(manifest, "_typeshed", "SupportsRead") == "unconstrained"
assert module._generated_protocol_status(manifest, "_typeshed", "SupportsWrite") == "unconstrained"
# A callable-level TypeVar that is not owned by the protocol stays unsupported.
assert module._generated_protocol_status(manifest, "_typeshed", "IdentityFunction") == "unsupported"
assert module._generated_protocol_status(manifest, "typing", "SupportsRound") == "unconstrained"
assert module._generated_protocol_status(manifest, "getopt", "_SliceableT") == "supported"
assert module._generated_protocol_status(manifest, "_typeshed", "DataclassInstance") == "unsupported"
assert module._generated_protocol_status(manifest, "_typeshed", "SupportsFlush") == "unconstrained"
assert module._generated_protocol_status(manifest, "http.server", "_SSLModule") == "unsupported"
assert module._generated_protocol_status(manifest, "xmlrpc.server", "_DispatchArityN") == "unsupported"

sigs = module.parse_generated_signature_param_index()
fixture = module.TYPE_DIR / "std-libs/_curses/getwin__file_as_SupportsRead_wrong.py"
assert module.unenforceable_generated_param_reason(fixture, sigs) is None
overload_fixture = module.TYPE_DIR / "std-libs/getopt/getopt__args_as__SliceableT_wrong.py"
assert module.unenforceable_generated_param_reason(overload_fixture, sigs) is None
"#;
    let output = Command::new("python3.12")
        .arg("-c")
        .arg(script)
        .current_dir(mamba_root())
        .output()
        .expect("run protocol accounting smoke");
    assert!(
        output.status.success(),
        "protocol accounting smoke failed\nstdout={}\nstderr={}",
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
fn partial_overload_accounting_binds_the_fixture_call_shape() {
    let script = r#"
import collections
import importlib.util
import pathlib
import sys
import tempfile

tool = pathlib.Path("tests/harness/cpython/tools/strict_type_accounting.py")
sys.path.insert(0, str(tool.parent))
spec = importlib.util.spec_from_file_location("strict_type_accounting", tool)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = module
spec.loader.exec_module(module)

sigs = module.parse_generated_signature_param_index()
assert sigs[("builtins", "list", "__getitem__")]["params"] == {
    "i": "partial",
    "s": "partial",
}

cases = [
    (
        "builtin-libs/builtins/list____getitem____i_as_SupportsIndex_wrong.py",
        "bound", ("pos", 0), "structured_param_partial",
    ),
    (
        "builtin-libs/builtins/frozenset____new____iterable_as_Iterable_wrong.py",
        "class", ("pos", 1), "structured_param_partial",
    ),
    (
        "std-libs/_sqlite3/adapt__alt_as__T_wrong.py",
        "module", ("pos", 2), "structured_param_partial",
    ),
    (
        "std-libs/contextlib/nullcontext__init__enter_result_as__T_wrong.py",
        "constructor", ("pos", 0), "contract_unconstrained",
    ),
    (
        "std-libs/asyncio_tasks/gather__coro_or_future1_as__FutureLike_wrong.py",
        "module", ("pos", 0), "structured_param_partial",
    ),
    (
        "std-libs/_curses/window__chgat__num_as_int_wrong.py",
        "bound", ("pos", 0), "structured_param_partial",
    ),
    (
        "std-libs/_curses/window__getstr__n_as_int_wrong.py",
        "bound", ("pos", 0), "structured_param_partial",
    ),
]
for relative, access, target, expected_reason in cases:
    path = module.TYPE_DIR / relative
    call, _param = module.parse_type_fixture_subject(path)
    key = module.resolve_generated_sig_key(call, sigs)
    assert key is not None
    shape = module.parse_type_fixture_call_shape(path, key)
    assert shape is not None
    assert shape.access == access, (relative, shape)
    assert shape.target == target, (relative, shape)
    assert module.unenforceable_generated_param_reason(path, sigs) == expected_reason

paths = module.executable_type_fixtures(sorted(module.TYPE_DIR.rglob("*.py")))
partial_outcomes = collections.Counter()
for path in paths:
    parsed = module.parse_type_fixture_subject(path)
    if parsed is None:
        continue
    call, param = parsed
    key = module.resolve_generated_sig_key(call, sigs)
    if key is None or sigs[key]["params"].get(param) != "partial":
        continue
    reason = module.unenforceable_generated_param_reason(path, sigs)
    if reason is None:
        partial_outcomes["supported"] += 1
    elif reason == "contract_unconstrained":
        partial_outcomes["unconstrained"] += 1
    else:
        partial_outcomes[reason] += 1
assert partial_outcomes == {
    "supported": 8,
    "unconstrained": 2,
    "structured_param_partial": 109,
}

unconstrained, unresolved = module.partition_generated_contract_coverage(paths, sigs)
reasons = collections.Counter(item["reason"] for item in unresolved)
assert reasons["structured_param_partial"] == 109
assert "structured_param_missing" not in reasons
assert "structured_signature_missing" not in reasons
assert not any(reason.startswith("stale_typevar_unwalled_marker_") for reason in reasons)
assert {
    pathlib.Path(item["path"]).name
    for item in unresolved
    if item["reason"] == "structured_param_partial"
} >= {
    "slice____new____start_as__T1_wrong.py",
    "slice____new____start_as_typed_wrong.py",
    "EnumMeta____call____names_as_typed_wrong.py",
}
assert any(
    item["path"].endswith("nullcontext__init__enter_result_as__T_wrong.py")
    for item in unconstrained
)

with tempfile.TemporaryDirectory() as tmp:
    tmp = pathlib.Path(tmp)
    invalid = (
        (
            ("demo", "C", "method"),
            "from demo import C\nreceiver.method(_W())",
            "value",
        ),
        (
            ("demo", "", "method"),
            "from demo import method\nmethod(_W(), _W())",
            "value",
        ),
        (
            ("demo", "", "method"),
            "from demo import method\nobj.method(_W())",
            "value",
        ),
        (
            ("demo", "", "method"),
            "from demo import method\nmethod(_W())",
            "other",
        ),
        (
            ("demo", "", "method"),
            "from demo import method\nmethod(*[_W()])",
            "value",
        ),
        (
            ("demo", "", "method"),
            "from other import method\nmethod(_W())",
            "value",
        ),
        (
            ("demo", "C", "method"),
            "from demo import C\nfrom other import Other\nobj = Other()\nobj.method(_W())",
            "value",
        ),
        (
            ("demo", "", "method"),
            "from demo import method\nmethod = replacement\nmethod(_W())",
            "value",
        ),
        (
            ("demo", "C", "method"),
            "from demo import C\nC = replacement\nC.method(_W())",
            "value",
        ),
        (
            ("demo", "C", "method"),
            "from demo import C as Alias\nAlias.C.method(_W())",
            "value",
        ),
        (
            ("demo", "", "method"),
            "def load():\n    from demo import method\nmethod(_W())",
            "value",
        ),
        (
            ("demo", "", "method"),
            "if False:\n    from demo import method\nmethod(_W())",
            "value",
        ),
        (
            ("demo", "", "method"),
            "from demo import method\ndef load(x=(method := replacement)):\n    pass\nmethod(_W())",
            "value",
        ),
        (
            ("demo", "", "method"),
            "from demo import method\ndef load(x: (method := replacement)):\n    pass\nmethod(_W())",
            "value",
        ),
        (
            ("demo", "", "method"),
            "from demo import method\nclass Rebind:\n    global method\n    method = replacement\nmethod(_W())",
            "value",
        ),
        (
            ("demo", "", "method"),
            "from demo import method\n[(method := replacement) for _ in [0]]\nmethod(_W())",
            "value",
        ),
        (
            ("demo", "", "method"),
            "from demo import method\nfrom other import *\nmethod(_W())",
            "value",
        ),
        (
            ("demo", "", "method"),
            "from demo import method\nraise TypeError('setup')\nmethod(_W())",
            "value",
        ),
        (
            ("demo", "", "method"),
            "from demo import method\ntry:\n    raise TypeError('setup')\n    method(_W())\nexcept TypeError:\n    pass",
            "value",
        ),
        (
            ("demo", "", "method"),
            "from demo import method\ndef call():\n    method(_W())",
            "value",
        ),
        (
            ("demo", "", "method"),
            "method(_W())\nfrom demo import method",
            "value",
        ),
        (
            ("demo", "C", "method"),
            "from demo import C\nobj = Other.__new__(C)\nobj.method(_W())",
            "value",
        ),
        (
            ("demo", "C", "method"),
            "from demo import C\nobj = object.__new__(C, _W())\nobj.method(_W())",
            "value",
        ),
        (
            ("demo", "C", "method"),
            "from demo import C\nobj = C(_W())\nobj.method(_W())",
            "value",
        ),
        (
            ("demo", "C", "method"),
            "from demo import C\nobj = C()\n(obj := Other())\nobj.method(_W())",
            "value",
        ),
        (
            ("demo", "C", "method"),
            "from demo import C\ndef load():\n    obj = C()\nobj.method(_W())",
            "value",
        ),
        (
            ("demo", "C", "method"),
            "from demo import C\nobj.method(_W())\nobj = C()",
            "value",
        ),
        (
            ("demo", "", "method"),
            "from demo import method\nmethod(_W(), {'value': 0})",
            "other",
        ),
        (
            ("builtins", "", "len"),
            "match 1:\n    case len:\n        pass\nlen(_W())",
            "value",
        ),
    )
    for index, (key, operation, marker_param) in enumerate(invalid):
        fixture = tmp / f"ambiguous-{index}.py"
        fixture.write_text(
            '# subject = "demo.C.method(value: T)"\n'
            'class _W:\n    pass\n'
            f'{operation}  # {marker_param}: T <- wrong-typed\n',
            encoding="utf-8",
        )
        assert module.parse_type_fixture_call_shape(fixture, key) is None

    late_import = tmp / "late-import.py"
    late_import.write_text(
        '# subject = "demo.method(value: T)"\n'
        'class _W:\n    pass\n'
        'method(_W())  # value: T <- wrong-typed\n'
        'from demo import method\n',
        encoding="utf-8",
    )
    assert module.parse_type_fixture_call_shape(
        late_import, ("demo", "", "method")
    ) is None

    late_receiver = tmp / "late-receiver.py"
    late_receiver.write_text(
        '# subject = "demo.C.method(value: T)"\n'
        'class _W:\n    pass\n'
        'from demo import C\n'
        'obj.method(_W())  # value: T <- wrong-typed\n'
        'obj = C()\n',
        encoding="utf-8",
    )
    assert module.parse_type_fixture_call_shape(
        late_receiver, ("demo", "C", "method")
    ) is None

    rebound_object = tmp / "rebound-object.py"
    rebound_object.write_text(
        '# subject = "demo.C.method(value: T)"\n'
        'class _W:\n    pass\n'
        'from demo import C\n'
        'object = Other\n'
        'obj = object.__new__(C)\n'
        'obj.method(_W())  # value: T <- wrong-typed\n',
        encoding="utf-8",
    )
    assert module.parse_type_fixture_call_shape(
        rebound_object, ("demo", "C", "method")
    ) is None

    contextual_marker = tmp / "contextual-marker.py"
    contextual_marker.write_text(
        '# subject = "demo.method(value: T)"\n'
        'class _W:\n    pass\n'
        'from demo import method\n'
        'method(_W())  # unrelated value: T <- wrong-typed trailing\n',
        encoding="utf-8",
    )
    assert module.parse_type_fixture_call_shape(
        contextual_marker, ("demo", "", "method")
    ) is None

    active_sentinel = tmp / "active-sentinel.py"
    active_sentinel.write_text(
        '# subject = "demo.method(value: T)"\n'
        'class _W:\n'
        '    def __init__(self, required):\n'
        '        pass\n'
        'from demo import method\n'
        'method(_W())  # value: T <- wrong-typed\n',
        encoding="utf-8",
    )
    assert module.parse_type_fixture_call_shape(
        active_sentinel, ("demo", "", "method")
    ) is None

    trailing_typeerror = tmp / "trailing-typeerror.py"
    trailing_typeerror.write_text(
        '# subject = "demo.method(value: T)"\n'
        'class _W:\n    pass\n'
        'from demo import method\n'
        'try:\n'
        '    method(_W())  # value: T <- wrong-typed\n'
        '    raise TypeError("after")\n'
        'except TypeError:\n'
        '    print("typeerror:")\n',
        encoding="utf-8",
    )
    assert module.parse_type_fixture_call_shape(
        trailing_typeerror, ("demo", "", "method")
    ) is None

    forged_handler = tmp / "forged-handler.py"
    forged_handler.write_text(
        '# subject = "demo.method(value: T)"\n'
        'class _W:\n    pass\n'
        'from demo import method\n'
        'try:\n'
        '    method(_W())  # value: T <- wrong-typed\n'
        '    print("no_typeerror:")\n'
        'except Exception as e:\n'
        '    print("typeerror:", type(e).__name__)\n'
        'except BaseException as e:\n'
        '    print("setup_or_other:", type(e).__name__)\n',
        encoding="utf-8",
    )
    assert module.parse_type_fixture_call_shape(
        forged_handler, ("demo", "", "method")
    ) is None

    class_alias = tmp / "class-alias.py"
    class_alias.write_text(
        '# subject = "demo.C.method(value: T)"\n'
        'class _W:\n    pass\n'
        'from demo import C as Alias\n'
        'Alias.method(_W())  # value: T <- wrong-typed\n',
        encoding="utf-8",
    )
    class_alias_shape = module.parse_type_fixture_call_shape(
        class_alias, ("demo", "C", "method")
    )
    assert class_alias_shape is not None and class_alias_shape.access == "class"

    nested_alias = tmp / "nested-class-alias.py"
    nested_alias.write_text(
        '# subject = "demo.Outer.Inner.method(value: T)"\n'
        'class _W:\n    pass\n'
        'from demo import Outer as Alias\n'
        'Alias.Inner.method(_W())  # value: T <- wrong-typed\n',
        encoding="utf-8",
    )
    nested_alias_shape = module.parse_type_fixture_call_shape(
        nested_alias, ("demo", "Outer.Inner", "method")
    )
    assert nested_alias_shape is not None and nested_alias_shape.access == "class"

    keyword = tmp / "keyword.py"
    keyword.write_text(
        '# subject = "demo.method(value: T)"\n'
        'class _W:\n    pass\n'
        'from demo import method\n'
        'method(value=_W())  # value: T <- wrong-typed\n',
        encoding="utf-8",
    )
    shape = module.parse_type_fixture_call_shape(keyword, ("demo", "", "method"))
    assert shape is not None and shape.target == ("kw", "value")
    param = {
        "name": "value", "kind": "k", "has_default": False,
        "implicit_receiver": False, "status": "supported", "reason": None,
    }
    branch = {"kind": "m", "ordered_params": [param]}
    assert module._status_for_fixture_call({"branch_specs": [branch]}, shape) == (
        "supported", None,
    )
    renamed = {**param, "name": "other"}
    assert module._status_for_fixture_call(
        {"branch_specs": [{"kind": "m", "ordered_params": [renamed]}]}, shape
    ) is None
    assert module._status_for_fixture_call(
        {"branch_specs": [branch, branch]}, shape
    ) is None
    unconstrained = {**param, "status": "unconstrained"}
    assert module._status_for_fixture_call(
        {"branch_specs": [branch, {"kind": "m", "ordered_params": [unconstrained]}]},
        shape,
    ) is None
"#;
    assert_python_script(script, "fixture-specific overload accounting smoke");
}

#[test]
fn imported_alias_accounting_stays_limited_to_proven_generated_identities() {
    let script = r#"
import importlib.util
import pathlib
import sys

gen_tool = pathlib.Path("tests/harness/cpython/tools/type_wall_gen.py")
sys.path.insert(0, str(gen_tool.parent))
gen_spec = importlib.util.spec_from_file_location("type_wall_gen", gen_tool)
gen = importlib.util.module_from_spec(gen_spec)
assert gen_spec.loader is not None
sys.modules[gen_spec.name] = gen
gen_spec.loader.exec_module(gen)

def info(
    module,
    *,
    alias=(),
    classes=(),
    class_aliases=(),
    imports=None,
    explicit=(),
    stars=(),
):
    return {
        "module": module,
        "alias_decls": list(alias),
        "class_decls": list(classes),
        "class_aliases": list(class_aliases),
        "imports": {} if imports is None else imports,
        "explicit_reexports": set(explicit),
        "star_imports": list(stars),
        "all_names": None,
    }

source = info("source", alias=[{"qualifier": "", "name": "Canonical"}])
reexport = info(
    "reexport",
    imports={"Alias": ("source", "Canonical")},
    explicit={"Alias"},
)
assert gen._spec_resolve_alias_exports([source, reexport])[("reexport", "Alias")] == (
    "source", "Canonical"
)

other = info("other", alias=[{"qualifier": "", "name": "Alias"}])
ambiguous = info(
    "ambiguous",
    imports={"Alias": ("source", "Canonical")},
    explicit={"Alias"},
    stars=("other",),
)
assert ("ambiguous", "Alias") not in gen._spec_resolve_alias_exports(
    [source, other, ambiguous]
)

collision = info(
    "collision",
    classes=[{"qualifier": "Alias", "name": "Alias"}],
    imports={"Alias": ("source", "Canonical")},
    explicit={"Alias"},
    stars=("other",),
)
collision_infos = [source, other, collision]
alias_candidates = gen._spec_alias_export_candidates(collision_infos)
class_candidates = gen._spec_class_export_candidates(collision_infos)
assert len(alias_candidates[("collision", "Alias")]) == 2
assert class_candidates[("collision", "Alias")] == {("collision", "Alias")}
assert ("collision", "Alias") not in gen._spec_unique_exports(
    class_candidates,
    alias_candidates,
)
assert gen._spec_resolve_imported_identity(
    "collision",
    "Alias",
    {},
    {("collision", "Alias"): "Nominal"},
    class_candidates,
    alias_candidates,
) == ("collision", "Alias", "Imported")

accounting_tool = pathlib.Path("tests/harness/cpython/tools/strict_type_accounting.py")
accounting_spec = importlib.util.spec_from_file_location("strict_type_accounting", accounting_tool)
accounting = importlib.util.module_from_spec(accounting_spec)
assert accounting_spec.loader is not None
sys.modules[accounting_spec.name] = accounting
accounting_spec.loader.exec_module(accounting)

relative_paths = [
    "std-libs/_frozen_importlib_external/PathFinder__find_distributions__context_as_Context_wrong.py",
    "std-libs/ssl/SSLSocket__connect__addr_as__Address_wrong.py",
    "std-libs/ssl/SSLSocket__connect_ex__addr_as__Address_wrong.py",
    "std-libs/wsgiref_handlers/BaseHandler__error_output__environ_as_WSGIEnvironment_wrong.py",
    "std-libs/wsgiref_handlers/BaseHandler__run__application_as_WSGIApplication_wrong.py",
    "std-libs/wsgiref_simple_server/WSGIServer__set_app__application_as_typed_wrong.py",
    "std-libs/wsgiref_simple_server/demo_app__environ_as_WSGIEnvironment_wrong.py",
    "std-libs/wsgiref_util/application_uri__environ_as_WSGIEnvironment_wrong.py",
    "std-libs/wsgiref_util/guess_scheme__environ_as_WSGIEnvironment_wrong.py",
    "std-libs/wsgiref_util/request_uri__environ_as_WSGIEnvironment_wrong.py",
    "std-libs/wsgiref_util/setup_testing_defaults__environ_as_WSGIEnvironment_wrong.py",
    "std-libs/wsgiref_util/shift_path_info__environ_as_WSGIEnvironment_wrong.py",
    "std-libs/wsgiref_validate/validator__application_as_WSGIApplication_wrong.py",
]
paths = [accounting.TYPE_DIR / path for path in relative_paths]
assert len(paths) == 13
assert all(path.is_file() for path in paths), paths
sigs = accounting.parse_generated_signature_param_index()
assert all(accounting.unenforceable_generated_param_reason(path, sigs) is None for path in paths)
"#;
    assert_python_script(script, "proven imported alias accounting regression");
}

#[test]
fn productive_recursive_alias_accounting_matches_checker_contractiveness() {
    let script = r#"
import importlib.util
import pathlib
import sys
from collections import Counter

tool = pathlib.Path("tests/harness/cpython/tools/strict_type_accounting.py")
sys.path.insert(0, str(tool.parent))
spec = importlib.util.spec_from_file_location("strict_type_accounting", tool)
accounting = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = accounting
spec.loader.exec_module(accounting)

manifest = {
    "strings": [""],
    "nodes": [],
    "edges": [],
    "aliases": [],
    "type_params": [],
    "type_param_edges": [],
}

def string_id(value):
    if value not in manifest["strings"]:
        manifest["strings"].append(value)
    return manifest["strings"].index(value)

def node(value):
    manifest["nodes"].append(value)
    return len(manifest["nodes"]) - 1

def edge_range(items):
    start = len(manifest["edges"])
    manifest["edges"].extend(items)
    return [start, len(items)]

def name(module, value, kind):
    return node({"Name": {
        "module": string_id(module), "name": string_id(value), "kind": kind,
    }})

def alias_ref(value):
    return name("example", value, "a")

def apply(base, args):
    return node({"Apply": {"base": base, "args": edge_range(args)}})

def union(items):
    return node({"Union": edge_range(items)})

def tuple_node(items):
    return node({"Tuple": edge_range(items)})

def param_list(items):
    return node({"ParamList": edge_range(items)})

def declare_alias(value, target, type_params=()):
    start = len(manifest["type_param_edges"])
    manifest["type_param_edges"].extend(type_params)
    manifest["aliases"].append({
        "module": string_id("example"),
        "name": string_id(value),
        "qualifier": 0,
        "target": target,
        "type_params": [start, len(type_params)],
    })

list_type = name("builtins", "list", "b")
int_type = name("builtins", "int", "b")
optional = name("typing", "Optional", "s")
typing_union = name("typing", "Union", "s")
callable_type = name("typing", "Callable", "s")
type_guard = name("typing", "TypeGuard", "s")
type_is = name("typing", "TypeIs", "s")
any_node = node("Any")
unsupported_node = node({"Unsupported": string_id("synthetic")})

cases = {}
direct = alias_ref("Direct")
declare_alias("Direct", direct)
cases["direct"] = (direct, "unsupported")
cases["outer_guard_does_not_fix_direct"] = (
    apply(list_type, [direct]), "unsupported",
)

guarded = alias_ref("Guarded")
declare_alias("Guarded", apply(list_type, [guarded]))
cases["list_guard"] = (guarded, "supported")

tuple_guarded = alias_ref("TupleGuarded")
declare_alias("TupleGuarded", tuple_node([tuple_guarded]))
cases["tuple_guard"] = (tuple_guarded, "supported")

callable_guarded = alias_ref("CallableGuarded")
declare_alias(
    "CallableGuarded",
    apply(callable_type, [param_list([callable_guarded]), int_type]),
)
cases["callable_guard"] = (callable_guarded, "supported")

optional_cycle = alias_ref("OptionalCycle")
declare_alias("OptionalCycle", apply(optional, [optional_cycle]))
cases["optional_is_transparent"] = (optional_cycle, "unsupported")

union_cycle = alias_ref("UnionCycle")
declare_alias("UnionCycle", apply(typing_union, [union_cycle, int_type]))
cases["typing_union_is_transparent"] = (union_cycle, "unsupported")

branch_cycle = alias_ref("BranchCycle")
declare_alias(
    "BranchCycle",
    union([apply(list_type, [branch_cycle]), branch_cycle]),
)
cases["guards_do_not_leak_between_union_branches"] = (
    branch_cycle, "unsupported",
)

optional_productive = alias_ref("OptionalProductive")
declare_alias(
    "OptionalProductive",
    apply(optional, [apply(list_type, [optional_productive])]),
)
cases["optional_can_contain_a_real_guard"] = (
    optional_productive, "supported",
)

for module, wrapper in [
    ("typing", "Annotated"),
    ("typing", "ClassVar"),
    ("typing", "Final"),
    ("typing", "Required"),
    ("typing", "NotRequired"),
    ("typing_extensions", "ReadOnly"),
]:
    alias_name = f"{wrapper}Cycle"
    recursive = alias_ref(alias_name)
    args = [recursive, any_node] if wrapper == "Annotated" else [recursive]
    declare_alias(alias_name, apply(name(module, wrapper, "s"), args))
    cases[f"{wrapper}_is_transparent"] = (recursive, "unsupported")

mutual_a = alias_ref("MutualA")
mutual_b = alias_ref("MutualB")
declare_alias("MutualA", mutual_b)
declare_alias("MutualB", mutual_a)
cases["mutual_direct"] = (mutual_a, "unsupported")

productive_a = alias_ref("ProductiveA")
productive_b = alias_ref("ProductiveB")
declare_alias("ProductiveA", apply(list_type, [productive_b]))
declare_alias("ProductiveB", productive_a)
cases["mutual_productive"] = (productive_a, "supported")

bad_a = alias_ref("BadA")
bad_b = alias_ref("BadB")
declare_alias("BadA", apply(list_type, [bad_b]))
declare_alias("BadB", bad_b)
cases["new_alias_frame_starts_unguarded"] = (bad_a, "unsupported")

manifest["type_params"].append({
    "constraints": [0, 0], "bound": None, "default": None,
    "kind": "t", "key": 0, "name": string_id("T"), "variance": "i",
})
type_param = node({"TypeParam": 0})
generic_a = alias_ref("GenericA")
generic_b = alias_ref("GenericB")
declare_alias("GenericA", apply(generic_b, [generic_a]))
declare_alias("GenericB", apply(list_type, [type_param]), [0])
cases["generic_substitution_keeps_guard"] = (generic_a, "supported")

identity_a = alias_ref("IdentityA")
identity_b = alias_ref("IdentityB")
declare_alias("IdentityA", apply(identity_b, [identity_a]))
declare_alias("IdentityB", type_param, [0])
cases["generic_identity_is_unproductive"] = (identity_a, "unsupported")

regular_transformed = alias_ref("RegularTransformed")
declare_alias(
    "RegularTransformed",
    apply(list_type, [apply(regular_transformed, [type_param])]),
    [0],
)
cases["regular_generic_recursion_keeps_structural_guard"] = (
    apply(regular_transformed, [int_type]), "supported",
)

transformed = alias_ref("Transformed")
declare_alias(
    "Transformed",
    apply(
        list_type,
        [apply(transformed, [apply(list_type, [type_param])])],
    ),
    [0],
)
cases["parameter_changing_recursion_is_fail_closed"] = (
    apply(transformed, [int_type]), "unsupported",
)

transformed_direct = alias_ref("TransformedDirect")
declare_alias(
    "TransformedDirect",
    apply(transformed_direct, [apply(list_type, [type_param])]),
    [0],
)
cases["parameter_changing_direct_cycle_is_unproductive"] = (
    apply(transformed_direct, [int_type]), "unsupported",
)

cases["type_guard_is_terminal"] = (
    apply(type_guard, [unsupported_node]), "supported",
)
cases["type_is_is_terminal"] = (
    apply(type_is, [unsupported_node]), "supported",
)

for label, (node_id, expected) in cases.items():
    actual = accounting._generated_typespec_status(manifest, node_id)
    assert actual == expected, (label, expected, actual)

live_manifest = accounting.load_generated_typespec_manifest()
for key in [
    ("marshal", "_Marshallable"),
    ("xmlrpc.client", "_Marshallable"),
    ("_typeshed", "TraceFunction"),
    ("xml.etree.ElementTree", "_ElementCallable"),
]:
    decl = accounting._generated_alias_decl(live_manifest, *key)
    assert decl is not None
    assert accounting._generated_typespec_status(
        live_manifest, decl["target"], alias_frames={key: False}
    ) == "supported", key

relative_paths = [
    "std-libs/marshal/dump__value_as__Marshallable_wrong.py",
    "std-libs/marshal/dumps__value_as__Marshallable_wrong.py",
    "std-libs/sys/settrace__function_as_typed_wrong.py",
    "std-libs/threading/settrace__func_as_typed_wrong.py",
    "std-libs/threading/settrace_all_threads__func_as_typed_wrong.py",
    "std-libs/xml_etree_ElementTree/Element__init__tag_as__Tag_wrong.py",
    "std-libs/xml_etree_ElementTree/Element__makeelement__tag_as__OtherTag_wrong.py",
    "std-libs/xmlrpc_client/Marshaller__dump_array__value_as_Iterable_wrong.py",
    "std-libs/xmlrpc_client/Marshaller__dump_struct__value_as_Mapping_wrong.py",
    "std-libs/xmlrpc_client/Marshaller__dumps__values_as_typed_wrong.py",
    "std-libs/xmlrpc_client/MultiCallIterator__init__results_as_list_wrong.py",
    "std-libs/xmlrpc_client/dumps__params_as_typed_wrong.py",
    "std-libs/xmlrpc_server/SimpleXMLRPCDispatcher__system_multicall__call_list_as_list_wrong.py",
]
paths = [accounting.TYPE_DIR / path for path in relative_paths]
assert len(paths) == 13
assert all(path.is_file() for path in paths), paths
sigs = accounting.parse_generated_signature_param_index()
assert all(
    accounting.unenforceable_generated_param_reason(path, sigs) is None
    for path in paths
)

expanded = Counter(
    status for row in sigs.values() for status in row["params"].values()
)
assert expanded == {
    "supported": 21328,
    "unconstrained": 2192,
    "partial": 1686,
    "unsupported": 1018,
}
wall = accounting.executable_type_fixtures(
    sorted(accounting.TYPE_DIR.rglob("*.py"))
)
unconstrained, unresolved = accounting.partition_generated_contract_coverage(
    wall, sigs
)
assert len(wall) == 7415
assert len(unconstrained) == 266
assert len(unresolved) == 152
assert Counter(item["reason"] for item in unresolved) == {
    "structured_param_partial": 109,
    "structured_param_type_unsupported": 43,
}
"#;
    assert_python_script(script, "productive recursive alias accounting regression");
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
fn optional_stdlib_extension_type_fixtures_follow_oracle_capabilities() {
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

strict_tk_fixture = strict_module.TYPE_DIR / "std-libs/_tkinter/TkappType__adderrorinfo__msg_as_str_wrong.py"
strict_ttk_fixture = strict_module.TYPE_DIR / "std-libs/tkinter_ttk/Button__configure__cnf_as_str_wrong.py"
strict_turtle_fixture = strict_module.TYPE_DIR / "std-libs/turtle/bgcolor__color_as__Color_wrong.py"
oracle_tk_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/_tkinter/TkappType__adderrorinfo__msg_as_str_wrong.py"
oracle_ttk_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/tkinter_ttk/Button__configure__cnf_as_str_wrong.py"
oracle_turtle_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/turtle/bgcolor__color_as__Color_wrong.py"

expected = importlib.util.find_spec("_tkinter") is None
assert strict_module.OPTIONAL_STDLIB_EXTENSION_TYPE_LIBS["_tkinter"] == "_tkinter"
assert strict_module.OPTIONAL_STDLIB_EXTENSION_TYPE_LIBS["tkinter_ttk"] == "_tkinter"
assert strict_module.OPTIONAL_STDLIB_EXTENSION_TYPE_LIBS["turtle"] == "_tkinter"
assert oracle_module.OPTIONAL_STDLIB_EXTENSION_TYPE_LIBS["_tkinter"] == "_tkinter"
assert oracle_module.OPTIONAL_STDLIB_EXTENSION_TYPE_LIBS["tkinter_ttk"] == "_tkinter"
assert oracle_module.OPTIONAL_STDLIB_EXTENSION_TYPE_LIBS["turtle"] == "_tkinter"
for strict_fixture, oracle_fixture in [
    (strict_tk_fixture, oracle_tk_fixture),
    (strict_ttk_fixture, oracle_ttk_fixture),
    (strict_turtle_fixture, oracle_turtle_fixture),
]:
    assert strict_module.is_optional_stdlib_extension_unavailable_type_fixture(strict_fixture) == expected
    assert oracle_module.is_optional_stdlib_extension_unavailable_type_fixture(oracle_fixture) == expected
    if expected:
        assert strict_fixture not in strict_module.executable_type_fixtures([strict_fixture])
"#;
    let output = Command::new("python3.12")
        .arg("-c")
        .arg(script)
        .current_dir(mamba_root())
        .output()
        .expect("run optional stdlib extension fixture smoke");
    assert!(
        output.status.success(),
        "optional stdlib extension fixture smoke failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn imported_identity_type_fixtures_remain_in_strict_type_accounting_wall() {
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

sigs = module.parse_generated_signature_param_index()
fixtures = [
    "tests/cpython/type/std-libs/importlib_metadata/DistributionFinder__find_distributions__context_as_Context_wrong.py",
    "tests/cpython/type/std-libs/_frozen_importlib_external/PathFinder__find_distributions__context_as_Context_wrong.py",
    "tests/cpython/type/std-libs/ssl/SSLSocket__connect__addr_as__Address_wrong.py",
    "tests/cpython/type/std-libs/ssl/SSLSocket__connect_ex__addr_as__Address_wrong.py",
    "tests/cpython/type/std-libs/wsgiref_handlers/BaseHandler__run__application_as_WSGIApplication_wrong.py",
    "tests/cpython/type/std-libs/wsgiref_util/application_uri__environ_as_WSGIEnvironment_wrong.py",
    "tests/cpython/type/std-libs/wsgiref_validate/validator__application_as_WSGIApplication_wrong.py",
]
for fixture in fixtures:
    path = pathlib.Path(fixture)
    subject = module.parse_type_fixture_subject(path)
    assert subject is not None, fixture
    key = module.resolve_generated_sig_key(subject[0], sigs)
    assert key is not None, fixture
    assert module.unenforceable_generated_param_reason(path, sigs) is None, fixture
"#;
    let output = run_python_script(script);
    assert!(
        output.status.success(),
        "imported identity accounting smoke failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn imported_identity_type_fixtures_still_reject_at_check_time() {
    let fixtures = [
        "tests/cpython/type/std-libs/importlib_metadata/DistributionFinder__find_distributions__context_as_Context_wrong.py",
        "tests/cpython/type/std-libs/_frozen_importlib_external/PathFinder__find_distributions__context_as_Context_wrong.py",
        "tests/cpython/type/std-libs/ssl/SSLSocket__connect__addr_as__Address_wrong.py",
        "tests/cpython/type/std-libs/ssl/SSLSocket__connect_ex__addr_as__Address_wrong.py",
        "tests/cpython/type/std-libs/wsgiref_handlers/BaseHandler__run__application_as_WSGIApplication_wrong.py",
        "tests/cpython/type/std-libs/wsgiref_validate/validator__application_as_WSGIApplication_wrong.py",
    ];
    for fixture in fixtures {
        let output = run_mamba_fixture(fixture);
        assert!(
            !output.status.success(),
            "fixture should reject at check time: {fixture}\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("type error") || stderr.contains("type mismatch"),
            "fixture should fail with a type rejection: {fixture}\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            stderr
        );
    }
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
strict_concurrent_interpreters_fixture = strict_module.TYPE_DIR / "std-libs/concurrent_interpreters/Interpreter__call__callable_as_Callable_wrong.py"
strict_concurrent_crossinterp_fixture = strict_module.TYPE_DIR / "std-libs/concurrent_interpreters__crossinterp/serialize_unbound__unbound_as__AnyUnbound_wrong.py"
strict_concurrent_queues_fixture = strict_module.TYPE_DIR / "std-libs/concurrent_interpreters__queues/Queue__get__block_as_bool_wrong.py"
strict_templatestr_fixture = strict_module.TYPE_DIR / "std-libs/ast/TemplateStr__init__values_as_list_wrong.py"
strict_z85decode_fixture = strict_module.TYPE_DIR / "std-libs/base64/z85decode__s_as_typed_wrong.py"
strict_z85encode_fixture = strict_module.TYPE_DIR / "std-libs/base64/z85encode__s_as_ReadableBuffer_wrong.py"
strict_date_strptime_date_string_fixture = strict_module.TYPE_DIR / "std-libs/datetime/date__strptime__date_string_as_str_wrong.py"
strict_date_strptime_string_fixture = strict_module.TYPE_DIR / "std-libs/datetime/date__strptime__string_as_str_wrong.py"
strict_time_strptime_date_string_fixture = strict_module.TYPE_DIR / "std-libs/datetime/time__strptime__date_string_as_str_wrong.py"
strict_time_strptime_string_fixture = strict_module.TYPE_DIR / "std-libs/datetime/time__strptime__string_as_str_wrong.py"
strict_asynchat_fixture = strict_module.TYPE_DIR / "std-libs/asynchat/async_chat__push__data_as_bytes_wrong.py"
strict_asyncio_coroutine_fixture = strict_module.TYPE_DIR / "std-libs/asyncio_coroutines/coroutine__func_as__FunctionT_wrong.py"
oracle_zstd_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/_zstd/finalize_dict__custom_dict_bytes_as_bytes_wrong.py"
oracle_annotationlib_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/annotationlib/ForwardRef__init__arg_as_str_wrong.py"
oracle_asyncio_graph_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/asyncio_graph/capture_call_graph__future_as_Future_wrong.py"
oracle_asyncio_tools_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/asyncio_tools/CycleFoundException__init__cycles_as_list_wrong.py"
oracle_concurrent_interpreters_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/concurrent_interpreters/Interpreter__call__callable_as_Callable_wrong.py"
oracle_concurrent_crossinterp_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/concurrent_interpreters__crossinterp/serialize_unbound__unbound_as__AnyUnbound_wrong.py"
oracle_concurrent_queues_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/concurrent_interpreters__queues/Queue__get__block_as_bool_wrong.py"
oracle_templatestr_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/ast/TemplateStr__init__values_as_list_wrong.py"
oracle_z85decode_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/base64/z85decode__s_as_typed_wrong.py"
oracle_z85encode_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/base64/z85encode__s_as_ReadableBuffer_wrong.py"
oracle_date_strptime_date_string_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/datetime/date__strptime__date_string_as_str_wrong.py"
oracle_date_strptime_string_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/datetime/date__strptime__string_as_str_wrong.py"
oracle_time_strptime_date_string_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/datetime/time__strptime__date_string_as_str_wrong.py"
oracle_time_strptime_string_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/datetime/time__strptime__string_as_str_wrong.py"
oracle_asynchat_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/asynchat/async_chat__push__data_as_bytes_wrong.py"
oracle_asyncio_coroutine_fixture = oracle_module.FIXTURES_ROOT / "type/std-libs/asyncio_coroutines/coroutine__func_as__FunctionT_wrong.py"
tkinter_fixture = strict_module.TYPE_DIR / "std-libs/_tkinter/TkappType__wantobjects__wantobjects_as_typed_wrong.py"

expected = sys.version_info[:2] < (3, 14)
expected_z85 = sys.version_info[:2] < (3, 13)
expected_removed = sys.version_info[:2] >= (3, 12)
assert strict_module.VERSION_SPECIFIC_TYPE_LIBS["annotationlib"] == (3, 14)
assert strict_module.VERSION_SPECIFIC_TYPE_LIBS["_zstd"] == (3, 14)
assert strict_module.VERSION_SPECIFIC_TYPE_LIBS["asyncio_graph"] == (3, 14)
assert strict_module.VERSION_SPECIFIC_TYPE_LIBS["asyncio_tools"] == (3, 14)
assert strict_module.VERSION_SPECIFIC_TYPE_LIBS["compression_zstd"] == (3, 14)
assert strict_module.VERSION_SPECIFIC_TYPE_LIBS["compression_zstd__zstdfile"] == (3, 14)
assert strict_module.VERSION_SPECIFIC_TYPE_LIBS["concurrent_interpreters"] == (3, 14)
assert strict_module.VERSION_SPECIFIC_TYPE_LIBS["concurrent_interpreters__crossinterp"] == (3, 14)
assert strict_module.VERSION_SPECIFIC_TYPE_LIBS["concurrent_interpreters__queues"] == (3, 14)
assert strict_module.VERSION_REMOVED_TYPE_LIBS["asynchat"] == (3, 12)
assert strict_module.VERSION_REMOVED_TYPE_LIBS["asyncore"] == (3, 12)
assert strict_module.VERSION_REMOVED_TYPE_LIBS["smtpd"] == (3, 12)
assert strict_module.VERSION_SPECIFIC_TYPE_FIXTURES["std-libs/ast/TemplateStr__init__values_as_list_wrong.py"] == (3, 14)
assert strict_module.VERSION_SPECIFIC_TYPE_FIXTURES["std-libs/base64/z85decode__s_as_typed_wrong.py"] == (3, 13)
assert strict_module.VERSION_SPECIFIC_TYPE_FIXTURES["std-libs/base64/z85encode__s_as_ReadableBuffer_wrong.py"] == (3, 13)
assert strict_module.VERSION_SPECIFIC_TYPE_FIXTURES["std-libs/datetime/date__strptime__date_string_as_str_wrong.py"] == (3, 14)
assert strict_module.VERSION_SPECIFIC_TYPE_FIXTURES["std-libs/datetime/date__strptime__string_as_str_wrong.py"] == (3, 14)
assert strict_module.VERSION_SPECIFIC_TYPE_FIXTURES["std-libs/datetime/time__strptime__date_string_as_str_wrong.py"] == (3, 14)
assert strict_module.VERSION_SPECIFIC_TYPE_FIXTURES["std-libs/datetime/time__strptime__string_as_str_wrong.py"] == (3, 14)
assert strict_module.VERSION_REMOVED_TYPE_FIXTURES["std-libs/asyncio_coroutines/coroutine__func_as__FunctionT_wrong.py"] == (3, 12)
assert oracle_module.VERSION_SPECIFIC_TYPE_LIBS["annotationlib"] == (3, 14)
assert oracle_module.VERSION_SPECIFIC_TYPE_LIBS["_zstd"] == (3, 14)
assert oracle_module.VERSION_SPECIFIC_TYPE_LIBS["asyncio_graph"] == (3, 14)
assert oracle_module.VERSION_SPECIFIC_TYPE_LIBS["asyncio_tools"] == (3, 14)
assert oracle_module.VERSION_SPECIFIC_TYPE_LIBS["concurrent_interpreters"] == (3, 14)
assert oracle_module.VERSION_SPECIFIC_TYPE_LIBS["concurrent_interpreters__crossinterp"] == (3, 14)
assert oracle_module.VERSION_SPECIFIC_TYPE_LIBS["concurrent_interpreters__queues"] == (3, 14)
assert oracle_module.VERSION_REMOVED_TYPE_LIBS["asynchat"] == (3, 12)
assert oracle_module.VERSION_REMOVED_TYPE_LIBS["asyncore"] == (3, 12)
assert oracle_module.VERSION_REMOVED_TYPE_LIBS["smtpd"] == (3, 12)
assert oracle_module.VERSION_SPECIFIC_TYPE_FIXTURES["std-libs/ast/TemplateStr__init__values_as_list_wrong.py"] == (3, 14)
assert oracle_module.VERSION_SPECIFIC_TYPE_FIXTURES["std-libs/base64/z85decode__s_as_typed_wrong.py"] == (3, 13)
assert oracle_module.VERSION_SPECIFIC_TYPE_FIXTURES["std-libs/base64/z85encode__s_as_ReadableBuffer_wrong.py"] == (3, 13)
assert oracle_module.VERSION_SPECIFIC_TYPE_FIXTURES["std-libs/datetime/date__strptime__date_string_as_str_wrong.py"] == (3, 14)
assert oracle_module.VERSION_SPECIFIC_TYPE_FIXTURES["std-libs/datetime/date__strptime__string_as_str_wrong.py"] == (3, 14)
assert oracle_module.VERSION_SPECIFIC_TYPE_FIXTURES["std-libs/datetime/time__strptime__date_string_as_str_wrong.py"] == (3, 14)
assert oracle_module.VERSION_SPECIFIC_TYPE_FIXTURES["std-libs/datetime/time__strptime__string_as_str_wrong.py"] == (3, 14)
assert oracle_module.VERSION_REMOVED_TYPE_FIXTURES["std-libs/asyncio_coroutines/coroutine__func_as__FunctionT_wrong.py"] == (3, 12)
assert strict_module.is_version_specific_unavailable_type_fixture(strict_annotationlib_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_zstd_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_compression_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_asyncio_graph_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_asyncio_tools_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_concurrent_interpreters_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_concurrent_crossinterp_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_concurrent_queues_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_templatestr_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_z85decode_fixture) == expected_z85
assert strict_module.is_version_specific_unavailable_type_fixture(strict_z85encode_fixture) == expected_z85
assert strict_module.is_version_specific_unavailable_type_fixture(strict_date_strptime_date_string_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_date_strptime_string_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_time_strptime_date_string_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_time_strptime_string_fixture) == expected
assert strict_module.is_version_specific_unavailable_type_fixture(strict_asynchat_fixture) == expected_removed
assert strict_module.is_version_specific_unavailable_type_fixture(strict_asyncio_coroutine_fixture) == expected_removed
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_annotationlib_fixture) == expected
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_zstd_fixture) == expected
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_asyncio_graph_fixture) == expected
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_asyncio_tools_fixture) == expected
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_concurrent_interpreters_fixture) == expected
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_concurrent_crossinterp_fixture) == expected
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_concurrent_queues_fixture) == expected
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_templatestr_fixture) == expected
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_z85decode_fixture) == expected_z85
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_z85encode_fixture) == expected_z85
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_date_strptime_date_string_fixture) == expected
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_date_strptime_string_fixture) == expected
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_time_strptime_date_string_fixture) == expected
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_time_strptime_string_fixture) == expected
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_asynchat_fixture) == expected_removed
assert oracle_module.is_version_specific_unavailable_type_fixture(oracle_asyncio_coroutine_fixture) == expected_removed
assert not strict_module.is_version_specific_unavailable_type_fixture(tkinter_fixture)
if expected:
    assert strict_annotationlib_fixture not in strict_module.executable_type_fixtures([strict_annotationlib_fixture])
    assert strict_zstd_fixture not in strict_module.executable_type_fixtures([strict_zstd_fixture])
    assert strict_asyncio_graph_fixture not in strict_module.executable_type_fixtures([strict_asyncio_graph_fixture])
    assert strict_asyncio_tools_fixture not in strict_module.executable_type_fixtures([strict_asyncio_tools_fixture])
    assert strict_concurrent_interpreters_fixture not in strict_module.executable_type_fixtures([strict_concurrent_interpreters_fixture])
    assert strict_concurrent_crossinterp_fixture not in strict_module.executable_type_fixtures([strict_concurrent_crossinterp_fixture])
    assert strict_concurrent_queues_fixture not in strict_module.executable_type_fixtures([strict_concurrent_queues_fixture])
    assert strict_templatestr_fixture not in strict_module.executable_type_fixtures([strict_templatestr_fixture])
    assert strict_date_strptime_date_string_fixture not in strict_module.executable_type_fixtures([strict_date_strptime_date_string_fixture])
    assert strict_date_strptime_string_fixture not in strict_module.executable_type_fixtures([strict_date_strptime_string_fixture])
    assert strict_time_strptime_date_string_fixture not in strict_module.executable_type_fixtures([strict_time_strptime_date_string_fixture])
    assert strict_time_strptime_string_fixture not in strict_module.executable_type_fixtures([strict_time_strptime_string_fixture])
if expected_z85:
    assert strict_z85decode_fixture not in strict_module.executable_type_fixtures([strict_z85decode_fixture])
    assert strict_z85encode_fixture not in strict_module.executable_type_fixtures([strict_z85encode_fixture])
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
fn configparser_parsingerror_filename_is_not_a_py312_strict_wall() {
    let root = mamba_root();
    assert!(
        !root
            .join(
                "tests/cpython/type/std-libs/configparser/ParsingError__filename__value_as_str_wrong.py"
            )
            .exists(),
        "CPython 3.12 has no callable ParsingError.filename(value) API; stale typeshed rows must not become executable strict walls"
    );

    let script = r#"
import configparser
import importlib.util
import pathlib
import sys

assert not hasattr(configparser.ParsingError, "filename")
err = configparser.ParsingError("source.ini")
assert not hasattr(err, "filename")
err.filename = 12345
assert err.filename == 12345

gen_tool = pathlib.Path("tests/harness/cpython/tools/type_wall_gen.py")
sys.path.insert(0, str(gen_tool.parent))
gen_spec = importlib.util.spec_from_file_location("type_wall_gen", gen_tool)
gen_module = importlib.util.module_from_spec(gen_spec)
assert gen_spec.loader is not None
sys.modules[gen_spec.name] = gen_module
gen_spec.loader.exec_module(gen_module)
assert gen_module.is_signature_param_not_wrongable("configparser", "ParsingError", "filename", "value")
"#;
    let output = Command::new("python3.12")
        .arg("-c")
        .arg(script)
        .current_dir(mamba_root())
        .output()
        .expect("run configparser ParsingError filename smoke");
    assert!(
        output.status.success(),
        "configparser ParsingError filename smoke failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn dataclasses_is_dataclass_query_is_not_a_strict_type_wall() {
    let root = mamba_root();
    for rel in [
        "is_dataclass__obj_as_Never_wrong.py",
        "is_dataclass__obj_as_type_wrong.py",
    ] {
        assert!(
            !root
                .join("tests/cpython/type/std-libs/dataclasses")
                .join(rel)
                .exists(),
            "dataclasses.is_dataclass(obj) is a query helper and must not be an executable strict wall: {rel}"
        );
    }

    let script = r#"
import dataclasses
import importlib.util
import pathlib
import sys

class Plain:
    pass

@dataclasses.dataclass
class Data:
    x: int = 1

assert dataclasses.is_dataclass(Plain()) is False
assert dataclasses.is_dataclass(Plain) is False
assert dataclasses.is_dataclass(Data()) is True
assert dataclasses.is_dataclass(Data) is True
assert dataclasses.is_dataclass(12345) is False
assert dataclasses.is_dataclass(None) is False

gen_tool = pathlib.Path("tests/harness/cpython/tools/type_wall_gen.py")
sys.path.insert(0, str(gen_tool.parent))
gen_spec = importlib.util.spec_from_file_location("type_wall_gen", gen_tool)
gen_module = importlib.util.module_from_spec(gen_spec)
assert gen_spec.loader is not None
sys.modules[gen_spec.name] = gen_module
gen_spec.loader.exec_module(gen_module)
assert gen_module.is_signature_param_not_wrongable("dataclasses", "", "is_dataclass", "obj")
"#;
    let output = Command::new("python3.12")
        .arg("-c")
        .arg(script)
        .current_dir(mamba_root())
        .output()
        .expect("run dataclasses is_dataclass smoke");
    assert!(
        output.status.success(),
        "dataclasses is_dataclass smoke failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
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
assert gen_module.is_signature_param_not_wrongable("configparser", "ParsingError", "filename", "value")
assert gen_module.is_signature_param_not_wrongable("dataclasses", "", "is_dataclass", "obj")
assert not gen_module.is_signature_param_not_wrongable("aifc", "Aifc_read", "getmark", "id")
assert not gen_module.is_signature_param_not_wrongable("argparse", "ArgumentParser", "error", "message")
assert not gen_module.is_signature_param_not_wrongable("array", "array", "__mul__", "value")

class_body = ast.parse('''
class Demo:
    @staticmethod
    def _static(value: int): ...
    @classmethod
    def _class(cls, value: int): ...
    @staticmethod
    def visible(value: int): ...
''').body[0].body
rows = list(gen_module._walk_class(class_body, "demo", "Demo", {"smethod"}))
assert [row["func"] for row in rows] == ["visible"]
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

    let root = mamba_root();
    let curses = root.join("tests/cpython/type/std-libs/_curses");
    for method in ["clearok", "idlok", "keypad", "leaveok", "nodelay", "notimeout"] {
        assert!(
            curses
                .join(format!("window__{method}__flag_as_bool_wrong.py"))
                .exists(),
            "canonical _curses {method}(flag: bool) fixture is missing"
        );
    }

    let copy = fs::read_to_string(
        root.join(
            "tests/cpython/type/std-libs/multiprocessing_sharedctypes/copy__obj_as__CT_wrong.py",
        ),
    )
    .expect("read sharedctypes.copy strict fixture");
    assert!(copy.contains("# mamba-strict-type: TypeError"));
    assert!(!copy.contains("TypeVar param must stay unwalled"));
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
    let root = mamba_root().join("src/types");
    let wrapper = fs::read_to_string(root.join("stdlib_specs_generated.rs"))
        .expect("read generated structured stdlib wrapper");
    assert!(wrapper.contains("schema: 2"));
    assert!(wrapper.contains("branches:"));
    assert!(wrapper.contains("type-nodes:"));
    assert!(wrapper.contains("class-callables:"));
    assert!(wrapper.contains("callable-exports:"));
    assert!(wrapper.contains("include_str!(\"stdlib_specs_generated.json\")"));

    let manifest = fs::read_to_string(root.join("stdlib_specs_generated.json"))
        .expect("read generated structured stdlib manifest");
    assert!(manifest.contains("\"schema\":2"));
    assert!(manifest.contains("\"callables\":"));
    assert!(manifest.contains("\"class_callables\":"));
    assert!(manifest.contains("\"class_exports\":"));
    assert!(manifest.contains("\"callable_exports\":"));
    assert!(manifest.contains("\"nodes\":"));
}

#[test]
fn typeshed_input_lock_matches_generated_provenance() {
    let script = r#"
import json
import pathlib
import tomllib

root = pathlib.Path(".")
lock = tomllib.loads(
    (root / "vendor/typeshed.lock.toml").read_text(encoding="utf-8")
)
manifest = json.loads(
    (root / "src/types/stdlib_specs_generated.json").read_text(encoding="utf-8")
)
expected = {
    "repository": lock["repository"],
    "revision": lock["revision"],
    "stdlib_pyi_count": lock["stdlib_pyi_count"],
    "stdlib_pyi_sha256": lock["stdlib_pyi_sha256"],
}
assert manifest["schema"] == 2
assert manifest["provenance"]["typeshed"] == expected

header_fields = {
    "typeshed-repository": expected["repository"],
    "typeshed-revision": expected["revision"],
    "typeshed-stdlib-pyi-count": expected["stdlib_pyi_count"],
    "typeshed-stdlib-pyi-sha256": expected["stdlib_pyi_sha256"],
}
for relative in (
    "src/types/stdlib_sigs_generated.rs",
    "src/types/stdlib_specs_generated.rs",
):
    text = (root / relative).read_text(encoding="utf-8")
    for name, value in header_fields.items():
        assert f"//! {name}: {value}" in text, (relative, name, value)
    assert "tests/harness/cpython/tools/checkout_typeshed.py" in text
"#;
    assert_python_script(script, "typeshed generated provenance smoke");
}

#[test]
fn typeshed_input_digest_is_stable_and_detects_drift() {
    let script = r##"
import pathlib
import sys
import tempfile

tools = pathlib.Path("tests/harness/cpython/tools").resolve()
sys.path.insert(0, str(tools))
import type_wall_gen
import wall_status
from typeshed_lock import (
    TypeshedLock,
    TypeshedLockError,
    load_typeshed_lock,
    stdlib_pyi_fingerprint,
    verify_typeshed_stdlib,
)

FIRST = b"def a() -> int: ...\n"
SECOND = b"class B: ...\n"
GOLDEN = "51df42981c4b4beec44587cf9614d55110e0c73f4f09d15dae073839dcf8b31f"

def write_corpus(root, reverse=False):
    rows = [(pathlib.Path("a.pyi"), FIRST), (pathlib.Path("nested/b.pyi"), SECOND)]
    if reverse:
        rows.reverse()
    for relative, content in rows:
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)

def expect_lock_error(call):
    try:
        call()
    except TypeshedLockError:
        return
    raise AssertionError("expected TypeshedLockError")

with tempfile.TemporaryDirectory() as tmp:
    tmp = pathlib.Path(tmp)
    first = tmp / "first"
    second = tmp / "second"
    write_corpus(first)
    write_corpus(second, reverse=True)
    first_fingerprint = stdlib_pyi_fingerprint(first)
    second_fingerprint = stdlib_pyi_fingerprint(second)
    assert first_fingerprint == second_fingerprint == (2, GOLDEN)

    lock = TypeshedLock(1, "https://example.invalid/typeshed.git", "a" * 40, 2, GOLDEN)
    verify_typeshed_stdlib(first, lock=lock)

    (first / "a.pyi").write_bytes(FIRST + b"# drift\n")
    expect_lock_error(lambda: verify_typeshed_stdlib(first, lock=lock))
    (first / "a.pyi").write_bytes(FIRST)
    (first / "extra.pyi").write_text("x: int\n", encoding="utf-8")
    expect_lock_error(lambda: verify_typeshed_stdlib(first, lock=lock))
    (first / "extra.pyi").unlink()
    (first / "nested/b.pyi").unlink()
    expect_lock_error(lambda: verify_typeshed_stdlib(first, lock=lock))

    malformed = [
        'schema = 2\nrepository = "https://example.invalid/typeshed.git"\nrevision = "' + "a" * 40 + '"\nstdlib_pyi_count = 2\nstdlib_pyi_sha256 = "' + GOLDEN + '"\n',
        'schema = 1\nrepository = "https://example.invalid/typeshed.git"\nrevision = "abc"\nstdlib_pyi_count = 2\nstdlib_pyi_sha256 = "' + GOLDEN + '"\n',
        'schema = 1\nrepository = "https://example.invalid/typeshed.git"\nrevision = "' + "a" * 40 + '"\nstdlib_pyi_count = 0\nstdlib_pyi_sha256 = "' + GOLDEN + '"\n',
        'schema = 1\nrepository = "https://example.invalid/typeshed.git"\nrevision = "' + "a" * 40 + '"\nstdlib_pyi_count = 2\nstdlib_pyi_sha256 = "abc"\n',
        'schema = 1\nrepository = "https://example.invalid/typeshed.git"\nrevision = "' + "A" * 40 + '"\nstdlib_pyi_count = 2\nstdlib_pyi_sha256 = "' + GOLDEN + '"\n',
        'schema = 1\nrepository = "https://example.invalid/typeshed.git"\nrevision = "' + "a" * 40 + '"\nstdlib_pyi_count = 2\nstdlib_pyi_sha256 = "' + GOLDEN + '"\nextra = true\n',
    ]
    for index, text in enumerate(malformed):
        path = tmp / f"bad-{index}.toml"
        path.write_text(text, encoding="utf-8")
        expect_lock_error(lambda path=path: load_typeshed_lock(path))

    drifted = tmp / "drifted"
    write_corpus(drifted)
    old = type_wall_gen.TYPESHED_STDLIB
    old_argv = sys.argv
    type_wall_gen.TYPESHED_STDLIB = drifted
    try:
        expect_lock_error(type_wall_gen.verify_typeshed_corpus)
        sys.argv = ["type_wall_gen.py", "--dry-run"]
        assert type_wall_gen.main() == 2
        expect_lock_error(wall_status.type_wall_signatures_and_cases)
    finally:
        sys.argv = old_argv
        type_wall_gen.TYPESHED_STDLIB = old
"##;
    assert_python_script(script, "typeshed fingerprint governance smoke");
}

#[test]
fn typeshed_checkout_is_exact_and_non_destructive() {
    let script = r#"
import pathlib
import shutil
import subprocess
import sys
import tempfile

tools = pathlib.Path("tests/harness/cpython/tools").resolve()
sys.path.insert(0, str(tools))
from checkout_typeshed import checkout_typeshed
from typeshed_lock import (
    TypeshedLock,
    TypeshedLockError,
    stdlib_pyi_fingerprint,
    verify_typeshed_stdlib,
)

def git(*args, cwd=None):
    result = subprocess.run(
        ["git", *args], cwd=cwd, check=True, text=True, capture_output=True
    )
    return result.stdout.strip()

def expect_lock_error(call):
    try:
        call()
    except TypeshedLockError:
        return
    raise AssertionError("expected TypeshedLockError")

with tempfile.TemporaryDirectory() as tmp:
    tmp = pathlib.Path(tmp)
    source = tmp / "source"
    source.mkdir()
    git("init", "--quiet", cwd=source)
    git("config", "user.name", "Mamba Test", cwd=source)
    git("config", "user.email", "mamba-test@example.invalid", cwd=source)
    (source / "stdlib").mkdir()
    (source / "stdlib/a.pyi").write_text("def a() -> int: ...\n", encoding="utf-8")
    (source / "stubs/demo").mkdir(parents=True)
    stub = source / "stubs/demo/demo.pyi"
    stub.write_text("VERSION: int\n", encoding="utf-8")
    git("add", ".", cwd=source)
    git("commit", "--quiet", "-m", "pinned", cwd=source)
    pinned_revision = git("rev-parse", "HEAD", cwd=source)
    count, digest = stdlib_pyi_fingerprint(source / "stdlib")

    stub.write_text("VERSION: str\n", encoding="utf-8")
    git("add", ".", cwd=source)
    git("commit", "--quiet", "-m", "moving head", cwd=source)
    moving_revision = git("rev-parse", "HEAD", cwd=source)
    assert moving_revision != pinned_revision

    lock = TypeshedLock(1, str(source), pinned_revision, count, digest)
    target = tmp / "target"
    checkout_typeshed(target, lock)
    assert git("rev-parse", "HEAD", cwd=target) == pinned_revision
    assert git("rev-parse", "--abbrev-ref", "HEAD", cwd=target) == "HEAD"
    assert git("remote", "get-url", "origin", cwd=target) == str(source)
    assert (target / "stubs/demo/demo.pyi").read_text(encoding="utf-8") == "VERSION: int\n"
    checkout_typeshed(target, lock)

    wrong_origin = str(tmp / "other-origin")
    git("remote", "set-url", "origin", wrong_origin, cwd=target)
    expect_lock_error(lambda: checkout_typeshed(target, lock))
    assert git("config", "--get", "remote.origin.url", cwd=target) == wrong_origin
    git("remote", "set-url", "origin", str(source), cwd=target)

    wrong = tmp / "wrong-revision"
    git("clone", "--quiet", str(source), str(wrong))
    assert git("rev-parse", "HEAD", cwd=wrong) == moving_revision
    expect_lock_error(lambda: checkout_typeshed(wrong, lock))
    assert git("rev-parse", "HEAD", cwd=wrong) == moving_revision

    dirty_stub = target / "stubs/demo/demo.pyi"
    dirty_stub.write_text("LOCAL: bool\n", encoding="utf-8")
    expect_lock_error(lambda: checkout_typeshed(target, lock))
    assert dirty_stub.read_text(encoding="utf-8") == "LOCAL: bool\n"

    non_git = tmp / "non-git"
    shutil.copytree(source / "stdlib", non_git / "stdlib")
    marker = non_git / "keep.txt"
    marker.write_text("preserve\n", encoding="utf-8")
    verify_typeshed_stdlib(non_git / "stdlib", lock=lock)
    expect_lock_error(lambda: checkout_typeshed(non_git, lock))
    assert marker.read_text(encoding="utf-8") == "preserve\n"
"#;
    assert_python_script(script, "typeshed checkout governance smoke");
}

#[test]
fn typeshed_acquisition_instructions_are_lock_aware() {
    let root = mamba_root();
    let helper = "tests/harness/cpython/tools/checkout_typeshed.py";
    let forbidden = "git clone --depth=1 https://github.com/python/typeshed.git";
    for relative in [
        ".gitignore",
        "llms.txt",
        "src/surface.rs",
        "src/types/stdlib_sigs.rs",
        "src/types/stdlib_sigs_generated.rs",
        "src/types/stdlib_specs_generated.rs",
    ] {
        let text = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|error| panic!("read {relative}: {error}"));
        assert!(
            !text.contains(forbidden),
            "{relative} must not instruct agents to clone moving typeshed HEAD"
        );
        assert!(
            text.contains(helper),
            "{relative} must route typeshed acquisition through {helper}"
        );
    }
}
