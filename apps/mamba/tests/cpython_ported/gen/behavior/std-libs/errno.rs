use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/errno/constants_are_distinct.py`.
#[test]
fn test_gen_behavior_std_libs_errno_constants_are_distinct() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "constants_are_distinct"
# subject = "errno.EACCES"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.EACCES: distinct named errors carry distinct values: EACCES != ENOENT"""
import errno

assert errno.EACCES != errno.ENOENT, (errno.EACCES, errno.ENOENT)
print("constants_are_distinct OK")
"###);
    assert_output(&out, r###"constants_are_distinct OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/errno/eagain_equals_ewouldblock.py`.
#[test]
fn test_gen_behavior_std_libs_errno_eagain_equals_ewouldblock() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "eagain_equals_ewouldblock"
# subject = "errno.EAGAIN"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.EAGAIN: EAGAIN and EWOULDBLOCK alias the same errno value (EAGAIN == EWOULDBLOCK)"""
import errno

assert errno.EAGAIN == errno.EWOULDBLOCK, (errno.EAGAIN, errno.EWOULDBLOCK)
print("eagain_equals_ewouldblock OK")
"###);
    assert_output(&out, r###"eagain_equals_ewouldblock OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/errno/errno_attribute_tests__test_for_improper_attributes.py`.
#[test]
fn test_gen_behavior_std_libs_errno_errno_attribute_tests__test_for_improper_attributes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "errno_attribute_tests__test_for_improper_attributes"
# subject = "cpython.test_errno.ErrnoAttributeTests.test_for_improper_attributes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_errno.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_errno.py::ErrnoAttributeTests::test_for_improper_attributes
"""Auto-ported test: ErrnoAttributeTests::test_for_improper_attributes (CPython 3.12 oracle)."""


import errno
import unittest


'Test the errno module\n   Roger E. Masse\n'

std_c_errors = frozenset(['EDOM', 'ERANGE'])


# --- test body ---
for error_code in std_c_errors:

    assert hasattr(errno, error_code)
print("ErrnoAttributeTests::test_for_improper_attributes: ok")
"###);
    assert_output(&out, r###"ErrnoAttributeTests::test_for_improper_attributes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/errno/errno_attribute_tests__test_using_errorcode.py`.
#[test]
fn test_gen_behavior_std_libs_errno_errno_attribute_tests__test_using_errorcode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "errno_attribute_tests__test_using_errorcode"
# subject = "cpython.test_errno.ErrnoAttributeTests.test_using_errorcode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_errno.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_errno.py::ErrnoAttributeTests::test_using_errorcode
"""Auto-ported test: ErrnoAttributeTests::test_using_errorcode (CPython 3.12 oracle)."""


import errno
import unittest


'Test the errno module\n   Roger E. Masse\n'

std_c_errors = frozenset(['EDOM', 'ERANGE'])


# --- test body ---
for value in errno.errorcode.values():

    assert hasattr(errno, value)
print("ErrnoAttributeTests::test_using_errorcode: ok")
"###);
    assert_output(&out, r###"ErrnoAttributeTests::test_using_errorcode: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/errno/errorcode_get_by_constant.py`.
#[test]
fn test_gen_behavior_std_libs_errno_errorcode_get_by_constant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "errorcode_get_by_constant"
# subject = "errno.errorcode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.errorcode: errorcode can be looked up by a named constant: errorcode.get(errno.EACCES) == 'EACCES' and errorcode[errno.ENOENT] == 'ENOENT'"""
import errno

assert errno.errorcode.get(errno.EACCES) == "EACCES", errno.errorcode.get(errno.EACCES)
assert errno.errorcode[errno.ENOENT] == "ENOENT", errno.errorcode[errno.ENOENT]
print("errorcode_get_by_constant OK")
"###);
    assert_output(&out, r###"errorcode_get_by_constant OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/errno/errorcode_get_unknown_is_none.py`.
#[test]
fn test_gen_behavior_std_libs_errno_errorcode_get_unknown_is_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "errorcode_get_unknown_is_none"
# subject = "errno.errorcode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.errorcode: errorcode.get on an errno number that is not present returns None (the dict .get default)"""
import errno

assert 99999 not in errno.errorcode, "99999 unexpectedly present in errorcode"
assert errno.errorcode.get(99999) is None, errno.errorcode.get(99999)
print("errorcode_get_unknown_is_none OK")
"###);
    assert_output(&out, r###"errorcode_get_unknown_is_none OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/errno/errorcode_key_value_types.py`.
#[test]
fn test_gen_behavior_std_libs_errno_errorcode_key_value_types() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "errorcode_key_value_types"
# subject = "errno.errorcode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.errorcode: every errorcode key is an int and every value is a str"""
import errno

assert errno.errorcode, "errorcode is unexpectedly empty"
assert all(isinstance(k, int) for k in errno.errorcode), "a key is not int"
assert all(isinstance(v, str) for v in errno.errorcode.values()), "a value is not str"
print("errorcode_key_value_types OK")
"###);
    assert_output(&out, r###"errorcode_key_value_types OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/errno/errorcode_maps_int_to_name.py`.
#[test]
fn test_gen_behavior_std_libs_errno_errorcode_maps_int_to_name() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "errorcode_maps_int_to_name"
# subject = "errno.errorcode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.errorcode: errorcode is a dict mapping the errno int to its uppercase name: errorcode[1]=='EPERM', errorcode[2]=='ENOENT', errorcode[13]=='EACCES'"""
import errno

assert errno.errorcode[1] == "EPERM", errno.errorcode[1]
assert errno.errorcode[2] == "ENOENT", errno.errorcode[2]
assert errno.errorcode[13] == "EACCES", errno.errorcode[13]
print("errorcode_maps_int_to_name OK")
"###);
    assert_output(&out, r###"errorcode_maps_int_to_name OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/errno/errorcode_values_are_attrs.py`.
#[test]
fn test_gen_behavior_std_libs_errno_errorcode_values_are_attrs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "errorcode_values_are_attrs"
# subject = "errno.errorcode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_errno.py"
# status = "filled"
# ///
"""errno.errorcode: every name in errorcode.values() is a real module attribute whose int round-trips back to the same errorcode key"""
import errno

for code, name in errno.errorcode.items():
    assert hasattr(errno, name), f"errorcode value {name!r} missing as attr"
    assert getattr(errno, name) == code, f"attr {name} != errorcode key {code}"
print("errorcode_values_are_attrs OK")
"###);
    assert_output(&out, r###"errorcode_values_are_attrs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/errno/posix_constant_values.py`.
#[test]
fn test_gen_behavior_std_libs_errno_posix_constant_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "posix_constant_values"
# subject = "errno.EPERM"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.EPERM: the POSIX-stable low constants pin to their documented numbers: EPERM==1, ENOENT==2, EBADF==9, EACCES==13"""
import errno

assert errno.EPERM == 1, errno.EPERM
assert errno.ENOENT == 2, errno.ENOENT
assert errno.EBADF == 9, errno.EBADF
assert errno.EACCES == 13, errno.EACCES
print("posix_constant_values OK")
"###);
    assert_output(&out, r###"posix_constant_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/errno/std_c_errors_are_int.py`.
#[test]
fn test_gen_behavior_std_libs_errno_std_c_errors_are_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "std_c_errors_are_int"
# subject = "errno.EDOM"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_errno.py"
# status = "filled"
# ///
"""errno.EDOM: the standard-C math errors EDOM and ERANGE are exposed as ints (not just POSIX errors are present)"""
import errno

for name in ("EDOM", "ERANGE"):
    assert hasattr(errno, name), f"errno is missing {name}"
    assert isinstance(getattr(errno, name), int), f"errno.{name} is not int"
print("std_c_errors_are_int OK")
"###);
    assert_output(&out, r###"std_c_errors_are_int OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/errno/uppercase_attrs_are_errorcode_keys.py`.
#[test]
fn test_gen_behavior_std_libs_errno_uppercase_attrs_are_errorcode_keys() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "uppercase_attrs_are_errorcode_keys"
# subject = "errno.errorcode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_errno.py"
# status = "filled"
# ///
"""errno.errorcode: every uppercase module attribute is an int whose value appears as a key in errorcode"""
import errno

upper_attrs = [a for a in errno.__dict__ if a.isupper()]
assert upper_attrs, "expected some uppercase errno constants"
for attr in upper_attrs:
    value = getattr(errno, attr)
    assert isinstance(value, int), f"{attr} should be int"
    assert value in errno.errorcode, f"{attr}={value} absent from errorcode"
print("uppercase_attrs_are_errorcode_keys OK")
"###);
    assert_output(&out, r###"uppercase_attrs_are_errorcode_keys OK
"###);
}
