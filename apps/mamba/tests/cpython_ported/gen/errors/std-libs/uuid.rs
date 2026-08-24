use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/uuid/bad_string_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_uuid_bad_string_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "errors"
# case = "bad_string_raises_valueerror"
# subject = "uuid.UUID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: bad_string_raises_valueerror (errors)."""
import uuid

_raised = False
try:
    uuid.UUID("not_a_uuid")
except ValueError:
    _raised = True
assert _raised, "bad_string_raises_valueerror: expected ValueError"
print("bad_string_raises_valueerror OK")
"###);
    assert_output(&out, r###"bad_string_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/uuid/bad_version_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_uuid_bad_version_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "errors"
# case = "bad_version_raises_valueerror"
# subject = "uuid.UUID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: bad_version_raises_valueerror (errors)."""
import uuid

_raised = False
try:
    uuid.UUID("12345678-1234-1234-1234-123456789012", version=99)
except ValueError:
    _raised = True
assert _raised, "bad_version_raises_valueerror: expected ValueError"
print("bad_version_raises_valueerror OK")
"###);
    assert_output(&out, r###"bad_version_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/uuid/multiple_constructor_args_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_uuid_multiple_constructor_args_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "errors"
# case = "multiple_constructor_args_raises_typeerror"
# subject = "uuid.UUID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: multiple_constructor_args_raises_typeerror (errors)."""
import uuid

_raised = False
try:
    uuid.UUID(hex="12345678-1234-1234-1234-123456789012", int=1)
except TypeError:
    _raised = True
assert _raised, "multiple_constructor_args_raises_typeerror: expected TypeError"
print("multiple_constructor_args_raises_typeerror OK")
"###);
    assert_output(&out, r###"multiple_constructor_args_raises_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/uuid/non_uuid_namespace_raises_attributeerror.py`.
#[test]
fn test_gen_errors_std_libs_uuid_non_uuid_namespace_raises_attributeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "errors"
# case = "non_uuid_namespace_raises_attributeerror"
# subject = "uuid.uuid5"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.uuid5: non_uuid_namespace_raises_attributeerror (errors)."""
import uuid

_raised = False
try:
    uuid.uuid5("not_uuid", "name")
except AttributeError:
    _raised = True
assert _raised, "non_uuid_namespace_raises_attributeerror: expected AttributeError"
print("non_uuid_namespace_raises_attributeerror OK")
"###);
    assert_output(&out, r###"non_uuid_namespace_raises_attributeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/uuid/set_attr_on_immutable_uuid_raises.py`.
#[test]
fn test_gen_errors_std_libs_uuid_set_attr_on_immutable_uuid_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "errors"
# case = "set_attr_on_immutable_uuid_raises"
# subject = "uuid.UUID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: set_attr_on_immutable_uuid_raises (errors)."""
import uuid

_raised = False
try:
    setattr(uuid.uuid4(), "hex", "x")
except TypeError:
    _raised = True
assert _raised, "set_attr_on_immutable_uuid_raises: expected TypeError"
print("set_attr_on_immutable_uuid_raises OK")
"###);
    assert_output(&out, r###"set_attr_on_immutable_uuid_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/uuid/short_bytes_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_uuid_short_bytes_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "errors"
# case = "short_bytes_raises_valueerror"
# subject = "uuid.UUID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: short_bytes_raises_valueerror (errors)."""
import uuid

_raised = False
try:
    uuid.UUID(bytes=bytes(4))
except ValueError:
    _raised = True
assert _raised, "short_bytes_raises_valueerror: expected ValueError"
print("short_bytes_raises_valueerror OK")
"###);
    assert_output(&out, r###"short_bytes_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/uuid/short_hex_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_uuid_short_hex_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "errors"
# case = "short_hex_raises_valueerror"
# subject = "uuid.UUID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: short_hex_raises_valueerror (errors)."""
import uuid

_raised = False
try:
    uuid.UUID("abc")
except ValueError:
    _raised = True
assert _raised, "short_hex_raises_valueerror: expected ValueError"
print("short_hex_raises_valueerror OK")
"###);
    assert_output(&out, r###"short_hex_raises_valueerror OK
"###);
}
