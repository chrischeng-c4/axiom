use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/struct/bad_type_for_int_code_raises.py`.
#[test]
fn test_gen_errors_std_libs_struct_bad_type_for_int_code_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "bad_type_for_int_code_raises"
# subject = "struct.pack"
# kind = "mechanical"
# xfail = "struct shim does not type-check pack args; accepts non-int silently (WI #3929)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: bad_type_for_int_code_raises (errors)."""
import struct

_raised = False
try:
    struct.pack("i", "not_int")
except struct.error:
    _raised = True
assert _raised, "bad_type_for_int_code_raises: expected struct.error"
print("bad_type_for_int_code_raises OK")
"###);
    assert_output(&out, r###"bad_type_for_int_code_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/struct/dangling_repeat_count_raises.py`.
#[test]
fn test_gen_errors_std_libs_struct_dangling_repeat_count_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "dangling_repeat_count_raises"
# subject = "struct.calcsize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.calcsize: dangling_repeat_count_raises (errors)."""
import struct

_raised = False
try:
    struct.calcsize("4")
except struct.error:
    _raised = True
assert _raised, "dangling_repeat_count_raises: expected struct.error"
print("dangling_repeat_count_raises OK")
"###);
    assert_output(&out, r###"dangling_repeat_count_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/struct/embedded_null_in_format_raises.py`.
#[test]
fn test_gen_errors_std_libs_struct_embedded_null_in_format_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "embedded_null_in_format_raises"
# subject = "struct.calcsize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
"""struct.calcsize: embedded_null_in_format_raises (errors)."""
import struct

_raised = False
try:
    struct.calcsize("2\u0000i")
except struct.error:
    _raised = True
assert _raised, "embedded_null_in_format_raises: expected struct.error"
print("embedded_null_in_format_raises OK")
"###);
    assert_output(&out, r###"embedded_null_in_format_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/struct/float_into_int_code_raises.py`.
#[test]
fn test_gen_errors_std_libs_struct_float_into_int_code_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "float_into_int_code_raises"
# subject = "struct.pack"
# kind = "mechanical"
# xfail = "struct shim does not reject a float for an int code; coerces/truncates (WI #3929)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: float_into_int_code_raises (errors)."""
import struct

_raised = False
try:
    struct.pack("i", 1.5)
except struct.error:
    _raised = True
assert _raised, "float_into_int_code_raises: expected struct.error"
print("float_into_int_code_raises OK")
"###);
    assert_output(&out, r###"float_into_int_code_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/struct/invalid_format_char_raises.py`.
#[test]
fn test_gen_errors_std_libs_struct_invalid_format_char_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "invalid_format_char_raises"
# subject = "struct.pack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: invalid_format_char_raises (errors)."""
import struct

_raised = False
try:
    struct.pack("z", 1)
except struct.error:
    _raised = True
assert _raised, "invalid_format_char_raises: expected struct.error"
print("invalid_format_char_raises OK")
"###);
    assert_output(&out, r###"invalid_format_char_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/struct/pack_value_out_of_range_raises.py`.
#[test]
fn test_gen_errors_std_libs_struct_pack_value_out_of_range_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "pack_value_out_of_range_raises"
# subject = "struct.pack"
# kind = "mechanical"
# xfail = "struct shim truncates instead of raising (WI #3929; struct_mod.rs has no range check)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: pack_value_out_of_range_raises (errors)."""
import struct

_raised = False
try:
    struct.pack("b", 1000)
except struct.error:
    _raised = True
assert _raised, "pack_value_out_of_range_raises: expected struct.error"
print("pack_value_out_of_range_raises OK")
"###);
    assert_output(&out, r###"pack_value_out_of_range_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/struct/stray_byteorder_marker_raises.py`.
#[test]
fn test_gen_errors_std_libs_struct_stray_byteorder_marker_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "stray_byteorder_marker_raises"
# subject = "struct.calcsize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.calcsize: stray_byteorder_marker_raises (errors)."""
import struct

_raised = False
try:
    struct.calcsize("i@i")
except struct.error:
    _raised = True
assert _raised, "stray_byteorder_marker_raises: expected struct.error"
print("stray_byteorder_marker_raises OK")
"###);
    assert_output(&out, r###"stray_byteorder_marker_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/struct/too_few_args_raises.py`.
#[test]
fn test_gen_errors_std_libs_struct_too_few_args_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "too_few_args_raises"
# subject = "struct.pack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: too_few_args_raises (errors)."""
import struct

_raised = False
try:
    struct.pack("ii", 1)
except struct.error:
    _raised = True
assert _raised, "too_few_args_raises: expected struct.error"
print("too_few_args_raises OK")
"###);
    assert_output(&out, r###"too_few_args_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/struct/too_many_args_raises.py`.
#[test]
fn test_gen_errors_std_libs_struct_too_many_args_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "too_many_args_raises"
# subject = "struct.pack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: too_many_args_raises (errors)."""
import struct

_raised = False
try:
    struct.pack("i", 1, 2, 3)
except struct.error:
    _raised = True
assert _raised, "too_many_args_raises: expected struct.error"
print("too_many_args_raises OK")
"###);
    assert_output(&out, r###"too_many_args_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/struct/trailing_repeat_count_raises.py`.
#[test]
fn test_gen_errors_std_libs_struct_trailing_repeat_count_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "trailing_repeat_count_raises"
# subject = "struct.pack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: trailing_repeat_count_raises (errors)."""
import struct

_raised = False
try:
    struct.pack("12345")
except struct.error:
    _raised = True
assert _raised, "trailing_repeat_count_raises: expected struct.error"
print("trailing_repeat_count_raises OK")
"###);
    assert_output(&out, r###"trailing_repeat_count_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/struct/unpack_non_buffer_type_error.py`.
#[test]
fn test_gen_errors_std_libs_struct_unpack_non_buffer_type_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "unpack_non_buffer_type_error"
# subject = "struct.unpack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
"""struct.unpack: unpack_non_buffer_type_error (errors)."""
import struct

_raised = False
try:
    struct.unpack("b", 0)
except TypeError:
    _raised = True
assert _raised, "unpack_non_buffer_type_error: expected TypeError"
print("unpack_non_buffer_type_error OK")
"###);
    assert_output(&out, r###"unpack_non_buffer_type_error OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/struct/unpack_wrong_size_raises.py`.
#[test]
fn test_gen_errors_std_libs_struct_unpack_wrong_size_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "unpack_wrong_size_raises"
# subject = "struct.unpack"
# kind = "mechanical"
# xfail = "struct shim does not validate buffer size; zero-pads/accepts instead of raising (WI #3929)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.unpack: unpack_wrong_size_raises (errors)."""
import struct

_raised = False
try:
    struct.unpack("ii", b"only4bytes")
except struct.error:
    _raised = True
assert _raised, "unpack_wrong_size_raises: expected struct.error"
print("unpack_wrong_size_raises OK")
"###);
    assert_output(&out, r###"unpack_wrong_size_raises OK
"###);
}
