use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/json/dumps_bytes_raises.py`.
#[test]
fn test_gen_errors_std_libs_json_dumps_bytes_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "errors"
# case = "dumps_bytes_raises"
# subject = "json.dumps"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_dump.py"
# status = "filled"
# ///
"""json.dumps: dumps_bytes_raises (errors)."""
import json

_raised = False
try:
    json.dumps(b"bytes are not json")
except TypeError:
    _raised = True
assert _raised, "dumps_bytes_raises: expected TypeError"
print("dumps_bytes_raises OK")
"###);
    assert_output(&out, r###"dumps_bytes_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/json/dumps_unserializable_object_raises.py`.
#[test]
fn test_gen_errors_std_libs_json_dumps_unserializable_object_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "errors"
# case = "dumps_unserializable_object_raises"
# subject = "json.dumps"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_dump.py"
# status = "filled"
# ///
"""json.dumps: dumps_unserializable_object_raises (errors)."""
import json

_raised = False
try:
    json.dumps(object())
except TypeError:
    _raised = True
assert _raised, "dumps_unserializable_object_raises: expected TypeError"
print("dumps_unserializable_object_raises OK")
"###);
    assert_output(&out, r###"dumps_unserializable_object_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/json/loads_bare_identifier_raises.py`.
#[test]
fn test_gen_errors_std_libs_json_loads_bare_identifier_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "errors"
# case = "loads_bare_identifier_raises"
# subject = "json.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_fail.py"
# status = "filled"
# ///
"""json.loads: loads_bare_identifier_raises (errors)."""
import json

_raised = False
try:
    json.loads("undefined")
except json.JSONDecodeError:
    _raised = True
assert _raised, "loads_bare_identifier_raises: expected json.JSONDecodeError"
print("loads_bare_identifier_raises OK")
"###);
    assert_output(&out, r###"loads_bare_identifier_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/json/loads_invalid_token_raises.py`.
#[test]
fn test_gen_errors_std_libs_json_loads_invalid_token_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "errors"
# case = "loads_invalid_token_raises"
# subject = "json.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_fail.py"
# status = "filled"
# ///
"""json.loads: loads_invalid_token_raises (errors)."""
import json

_raised = False
try:
    json.loads("{invalid}")
except json.JSONDecodeError:
    _raised = True
assert _raised, "loads_invalid_token_raises: expected json.JSONDecodeError"
print("loads_invalid_token_raises OK")
"###);
    assert_output(&out, r###"loads_invalid_token_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/json/loads_trailing_garbage_raises.py`.
#[test]
fn test_gen_errors_std_libs_json_loads_trailing_garbage_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "errors"
# case = "loads_trailing_garbage_raises"
# subject = "json.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_fail.py"
# status = "filled"
# ///
"""json.loads: loads_trailing_garbage_raises (errors)."""
import json

_raised = False
try:
    json.loads("{} extra")
except json.JSONDecodeError:
    _raised = True
assert _raised, "loads_trailing_garbage_raises: expected json.JSONDecodeError"
print("loads_trailing_garbage_raises OK")
"###);
    assert_output(&out, r###"loads_trailing_garbage_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/json/loads_truncated_object_raises.py`.
#[test]
fn test_gen_errors_std_libs_json_loads_truncated_object_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "errors"
# case = "loads_truncated_object_raises"
# subject = "json.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_fail.py"
# status = "filled"
# ///
"""json.loads: loads_truncated_object_raises (errors)."""
import json

_raised = False
try:
    json.loads("{")
except json.JSONDecodeError:
    _raised = True
assert _raised, "loads_truncated_object_raises: expected json.JSONDecodeError"
print("loads_truncated_object_raises OK")
"###);
    assert_output(&out, r###"loads_truncated_object_raises OK
"###);
}
