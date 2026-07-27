use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/array/append_overflow_overflowerror.py`.
#[test]
fn test_gen_errors_std_libs_array_append_overflow_overflowerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "append_overflow_overflowerror"
# subject = "array.array.append"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array.append: append_overflow_overflowerror (errors)."""
import array

_raised = False
try:
    array.array("b").append(1000)
except OverflowError:
    _raised = True
assert _raised, "append_overflow_overflowerror: expected OverflowError"
print("append_overflow_overflowerror OK")
"###);
    assert_output(&out, r###"append_overflow_overflowerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/array/append_type_mismatch_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_array_append_type_mismatch_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "append_type_mismatch_typeerror"
# subject = "array.array.append"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array.append: append_type_mismatch_typeerror (errors)."""
import array

_raised = False
try:
    array.array("i", [1, 2, 3]).append("x")
except TypeError:
    _raised = True
assert _raised, "append_type_mismatch_typeerror: expected TypeError"
print("append_type_mismatch_typeerror OK")
"###);
    assert_output(&out, r###"append_type_mismatch_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/array/bad_typecode_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_array_bad_typecode_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "bad_typecode_valueerror"
# subject = "array.array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: bad_typecode_valueerror (errors)."""
import array

_raised = False
try:
    array.array("Z")
except ValueError:
    _raised = True
assert _raised, "bad_typecode_valueerror: expected ValueError"
print("bad_typecode_valueerror OK")
"###);
    assert_output(&out, r###"bad_typecode_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/array/frombytes_nonmultiple_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_array_frombytes_nonmultiple_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "frombytes_nonmultiple_valueerror"
# subject = "array.array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array: frombytes_nonmultiple_valueerror (errors)."""
import array

_raised = False
try:
    array.array("i", b"12345")
except ValueError:
    _raised = True
assert _raised, "frombytes_nonmultiple_valueerror: expected ValueError"
print("frombytes_nonmultiple_valueerror OK")
"###);
    assert_output(&out, r###"frombytes_nonmultiple_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/array/getitem_out_of_range_indexerror.py`.
#[test]
fn test_gen_errors_std_libs_array_getitem_out_of_range_indexerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "getitem_out_of_range_indexerror"
# subject = "array.array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array: getitem_out_of_range_indexerror (errors)."""
import array

_raised = False
try:
    array.array("i", [1, 2, 3])[99]
except IndexError:
    _raised = True
assert _raised, "getitem_out_of_range_indexerror: expected IndexError"
print("getitem_out_of_range_indexerror OK")
"###);
    assert_output(&out, r###"getitem_out_of_range_indexerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/array/index_missing_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_array_index_missing_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "index_missing_valueerror"
# subject = "array.array.index"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.index: index_missing_valueerror (errors)."""
import array

_raised = False
try:
    array.array("i", [1, 2, 3]).index(99)
except ValueError:
    _raised = True
assert _raised, "index_missing_valueerror: expected ValueError"
print("index_missing_valueerror OK")
"###);
    assert_output(&out, r###"index_missing_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/array/int_array_rejects_float_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_array_int_array_rejects_float_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "int_array_rejects_float_typeerror"
# subject = "array.array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: int_array_rejects_float_typeerror (errors)."""
import array

_raised = False
try:
    array.array("i", [1, 2.5])
except TypeError:
    _raised = True
assert _raised, "int_array_rejects_float_typeerror: expected TypeError"
print("int_array_rejects_float_typeerror OK")
"###);
    assert_output(&out, r###"int_array_rejects_float_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/array/int_typecode_rejects_str_init_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_array_int_typecode_rejects_str_init_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "int_typecode_rejects_str_init_typeerror"
# subject = "array.array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array: int_typecode_rejects_str_init_typeerror (errors)."""
import array

_raised = False
try:
    array.array("b", "foo")
except TypeError:
    _raised = True
assert _raised, "int_typecode_rejects_str_init_typeerror: expected TypeError"
print("int_typecode_rejects_str_init_typeerror OK")
"###);
    assert_output(&out, r###"int_typecode_rejects_str_init_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/array/mixed_typecode_concat_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_array_mixed_typecode_concat_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "mixed_typecode_concat_typeerror"
# subject = "array.array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array: mixed_typecode_concat_typeerror (errors)."""
import array

_raised = False
try:
    array.array("i", [1]) + array.array("d", [1.0])
except TypeError:
    _raised = True
assert _raised, "mixed_typecode_concat_typeerror: expected TypeError"
print("mixed_typecode_concat_typeerror OK")
"###);
    assert_output(&out, r###"mixed_typecode_concat_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/array/multichar_typecode_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_array_multichar_typecode_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "multichar_typecode_typeerror"
# subject = "array.array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array: multichar_typecode_typeerror (errors)."""
import array

_raised = False
try:
    array.array("xx")
except TypeError:
    _raised = True
assert _raised, "multichar_typecode_typeerror: expected TypeError"
print("multichar_typecode_typeerror OK")
"###);
    assert_output(&out, r###"multichar_typecode_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/array/pop_empty_indexerror.py`.
#[test]
fn test_gen_errors_std_libs_array_pop_empty_indexerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "pop_empty_indexerror"
# subject = "array.array.pop"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.pop: pop_empty_indexerror (errors)."""
import array

_raised = False
try:
    array.array("i").pop()
except IndexError:
    _raised = True
assert _raised, "pop_empty_indexerror: expected IndexError"
print("pop_empty_indexerror OK")
"###);
    assert_output(&out, r###"pop_empty_indexerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/array/remove_missing_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_array_remove_missing_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "remove_missing_valueerror"
# subject = "array.array.remove"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.remove: remove_missing_valueerror (errors)."""
import array

_raised = False
try:
    array.array("i", [1, 2, 3]).remove(99)
except ValueError:
    _raised = True
assert _raised, "remove_missing_valueerror: expected ValueError"
print("remove_missing_valueerror OK")
"###);
    assert_output(&out, r###"remove_missing_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/array/wrong_init_type_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_array_wrong_init_type_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "wrong_init_type_typeerror"
# subject = "array.array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array: wrong_init_type_typeerror (errors)."""
import array

_raised = False
try:
    array.array("i", "abc")
except TypeError:
    _raised = True
assert _raised, "wrong_init_type_typeerror: expected TypeError"
print("wrong_init_type_typeerror OK")
"###);
    assert_output(&out, r###"wrong_init_type_typeerror OK
"###);
}
