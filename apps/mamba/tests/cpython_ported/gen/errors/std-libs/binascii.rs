use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/binascii/a2b_base64_incorrect_padding_raises.py`.
#[test]
fn test_gen_errors_std_libs_binascii_a2b_base64_incorrect_padding_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "errors"
# case = "a2b_base64_incorrect_padding_raises"
# subject = "binascii.a2b_base64"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_base64: a2b_base64_incorrect_padding_raises (errors)."""
import binascii

_raised = False
try:
    binascii.a2b_base64(b'abc')
except binascii.Error:
    _raised = True
assert _raised, "a2b_base64_incorrect_padding_raises: expected binascii.Error"
print("a2b_base64_incorrect_padding_raises OK")
"###);
    assert_output(&out, r###"a2b_base64_incorrect_padding_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/binascii/a2b_base64_invalid_length_raises.py`.
#[test]
fn test_gen_errors_std_libs_binascii_a2b_base64_invalid_length_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "errors"
# case = "a2b_base64_invalid_length_raises"
# subject = "binascii.a2b_base64"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_base64: a2b_base64_invalid_length_raises (errors)."""
import binascii

_raised = False
try:
    binascii.a2b_base64(b'a')
except binascii.Error:
    _raised = True
assert _raised, "a2b_base64_invalid_length_raises: expected binascii.Error"
print("a2b_base64_invalid_length_raises OK")
"###);
    assert_output(&out, r###"a2b_base64_invalid_length_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/binascii/a2b_base64_non_ascii_str_raises.py`.
#[test]
fn test_gen_errors_std_libs_binascii_a2b_base64_non_ascii_str_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "errors"
# case = "a2b_base64_non_ascii_str_raises"
# subject = "binascii.a2b_base64"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_base64: a2b_base64_non_ascii_str_raises (errors)."""
import binascii

_raised = False
try:
    binascii.a2b_base64('\x80')
except ValueError:
    _raised = True
assert _raised, "a2b_base64_non_ascii_str_raises: expected ValueError"
print("a2b_base64_non_ascii_str_raises OK")
"###);
    assert_output(&out, r###"a2b_base64_non_ascii_str_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/binascii/a2b_base64_strict_invalid_raises.py`.
#[test]
fn test_gen_errors_std_libs_binascii_a2b_base64_strict_invalid_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "errors"
# case = "a2b_base64_strict_invalid_raises"
# subject = "binascii.a2b_base64"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_base64: a2b_base64_strict_invalid_raises (errors)."""
import binascii

_raised = False
try:
    binascii.a2b_base64(b'not_b64!@#$', strict_mode=True)
except binascii.Error:
    _raised = True
assert _raised, "a2b_base64_strict_invalid_raises: expected binascii.Error"
print("a2b_base64_strict_invalid_raises OK")
"###);
    assert_output(&out, r###"a2b_base64_strict_invalid_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/binascii/a2b_hex_non_hex_char_raises.py`.
#[test]
fn test_gen_errors_std_libs_binascii_a2b_hex_non_hex_char_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "errors"
# case = "a2b_hex_non_hex_char_raises"
# subject = "binascii.a2b_hex"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_hex: a2b_hex_non_hex_char_raises (errors)."""
import binascii

_raised = False
try:
    binascii.a2b_hex(b'0G')
except binascii.Error:
    _raised = True
assert _raised, "a2b_hex_non_hex_char_raises: expected binascii.Error"
print("a2b_hex_non_hex_char_raises OK")
"###);
    assert_output(&out, r###"a2b_hex_non_hex_char_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/binascii/b2a_base64_str_input_raises.py`.
#[test]
fn test_gen_errors_std_libs_binascii_b2a_base64_str_input_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "errors"
# case = "b2a_base64_str_input_raises"
# subject = "binascii.b2a_base64"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_base64: b2a_base64_str_input_raises (errors)."""
import binascii

_raised = False
try:
    binascii.b2a_base64('text')
except TypeError:
    _raised = True
assert _raised, "b2a_base64_str_input_raises: expected TypeError"
print("b2a_base64_str_input_raises OK")
"###);
    assert_output(&out, r###"b2a_base64_str_input_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/binascii/b2a_uu_over_45_bytes_raises.py`.
#[test]
fn test_gen_errors_std_libs_binascii_b2a_uu_over_45_bytes_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "errors"
# case = "b2a_uu_over_45_bytes_raises"
# subject = "binascii.b2a_uu"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_uu: b2a_uu_over_45_bytes_raises (errors)."""
import binascii

_raised = False
try:
    binascii.b2a_uu(b'!' * 46)
except binascii.Error:
    _raised = True
assert _raised, "b2a_uu_over_45_bytes_raises: expected binascii.Error"
print("b2a_uu_over_45_bytes_raises OK")
"###);
    assert_output(&out, r###"b2a_uu_over_45_bytes_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/binascii/unhexlify_odd_length_raises.py`.
#[test]
fn test_gen_errors_std_libs_binascii_unhexlify_odd_length_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "errors"
# case = "unhexlify_odd_length_raises"
# subject = "binascii.unhexlify"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.unhexlify: unhexlify_odd_length_raises (errors)."""
import binascii

_raised = False
try:
    binascii.unhexlify(b'abc')
except binascii.Error:
    _raised = True
assert _raised, "unhexlify_odd_length_raises: expected binascii.Error"
print("unhexlify_odd_length_raises OK")
"###);
    assert_output(&out, r###"unhexlify_odd_length_raises OK
"###);
}
