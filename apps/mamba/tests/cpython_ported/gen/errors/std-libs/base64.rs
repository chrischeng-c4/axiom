use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/base64/a85decode_out_of_range_char_raises.py`.
#[test]
fn test_gen_errors_std_libs_base64_a85decode_out_of_range_char_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "a85decode_out_of_range_char_raises"
# subject = "base64.a85decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.a85decode: a85decode_out_of_range_char_raises (errors)."""
import base64

_raised = False
try:
    base64.a85decode(b'!!!!y')
except ValueError:
    _raised = True
assert _raised, "a85decode_out_of_range_char_raises: expected ValueError"
print("a85decode_out_of_range_char_raises OK")
"###);
    assert_output(&out, r###"a85decode_out_of_range_char_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/base64/b16decode_odd_length_raises.py`.
#[test]
fn test_gen_errors_std_libs_base64_b16decode_odd_length_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "b16decode_odd_length_raises"
# subject = "base64.b16decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b16decode: b16decode_odd_length_raises (errors)."""
import base64
import binascii

_raised = False
try:
    base64.b16decode(b'abc')
except binascii.Error:
    _raised = True
assert _raised, "b16decode_odd_length_raises: expected binascii.Error"
print("b16decode_odd_length_raises OK")
"###);
    assert_output(&out, r###"b16decode_odd_length_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/base64/b32decode_bad_chars_raises.py`.
#[test]
fn test_gen_errors_std_libs_base64_b32decode_bad_chars_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "b32decode_bad_chars_raises"
# subject = "base64.b32decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b32decode: b32decode_bad_chars_raises (errors)."""
import base64
import binascii

_raised = False
try:
    base64.b32decode(b'not_b32_chars!', casefold=False)
except binascii.Error:
    _raised = True
assert _raised, "b32decode_bad_chars_raises: expected binascii.Error"
print("b32decode_bad_chars_raises OK")
"###);
    assert_output(&out, r###"b32decode_bad_chars_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/base64/b64decode_invalid_chars_validate_raises.py`.
#[test]
fn test_gen_errors_std_libs_base64_b64decode_invalid_chars_validate_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "b64decode_invalid_chars_validate_raises"
# subject = "base64.b64decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b64decode: b64decode_invalid_chars_validate_raises (errors)."""
import base64
import binascii

_raised = False
try:
    base64.b64decode(b'not_base64!@#$', validate=True)
except binascii.Error:
    _raised = True
assert _raised, "b64decode_invalid_chars_validate_raises: expected binascii.Error"
print("b64decode_invalid_chars_validate_raises OK")
"###);
    assert_output(&out, r###"b64decode_invalid_chars_validate_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/base64/b64decode_odd_padding_raises.py`.
#[test]
fn test_gen_errors_std_libs_base64_b64decode_odd_padding_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "b64decode_odd_padding_raises"
# subject = "base64.b64decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b64decode: b64decode_odd_padding_raises (errors)."""
import base64
import binascii

_raised = False
try:
    base64.b64decode(b'abc')
except binascii.Error:
    _raised = True
assert _raised, "b64decode_odd_padding_raises: expected binascii.Error"
print("b64decode_odd_padding_raises OK")
"###);
    assert_output(&out, r###"b64decode_odd_padding_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/base64/b64decode_str_odd_padding_raises.py`.
#[test]
fn test_gen_errors_std_libs_base64_b64decode_str_odd_padding_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "b64decode_str_odd_padding_raises"
# subject = "base64.b64decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b64decode: b64decode_str_odd_padding_raises (errors)."""
import base64
import binascii

_raised = False
try:
    base64.b64decode('abc')
except binascii.Error:
    _raised = True
assert _raised, "b64decode_str_odd_padding_raises: expected binascii.Error"
print("b64decode_str_odd_padding_raises OK")
"###);
    assert_output(&out, r###"b64decode_str_odd_padding_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/base64/b85decode_illegal_char_raises.py`.
#[test]
fn test_gen_errors_std_libs_base64_b85decode_illegal_char_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "b85decode_illegal_char_raises"
# subject = "base64.b85decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b85decode: b85decode_illegal_char_raises (errors)."""
import base64

_raised = False
try:
    base64.b85decode(b'0000"')
except ValueError:
    _raised = True
assert _raised, "b85decode_illegal_char_raises: expected ValueError"
print("b85decode_illegal_char_raises OK")
"###);
    assert_output(&out, r###"b85decode_illegal_char_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/base64/b85decode_overflow_group_raises.py`.
#[test]
fn test_gen_errors_std_libs_base64_b85decode_overflow_group_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "b85decode_overflow_group_raises"
# subject = "base64.b85decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b85decode: b85decode_overflow_group_raises (errors)."""
import base64

_raised = False
try:
    base64.b85decode(b'|NsC1')
except ValueError:
    _raised = True
assert _raised, "b85decode_overflow_group_raises: expected ValueError"
print("b85decode_overflow_group_raises OK")
"###);
    assert_output(&out, r###"b85decode_overflow_group_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/base64/binascii_error_is_valueerror_subclass.py`.
#[test]
fn test_gen_errors_std_libs_base64_binascii_error_is_valueerror_subclass() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "binascii_error_is_valueerror_subclass"
# subject = "base64.b64decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b64decode: binascii.Error is a subclass of ValueError so callers can catch either; this is part of the public decode contract"""
import base64
import binascii

assert issubclass(binascii.Error, ValueError), "binascii.Error <: ValueError"
# Therefore a bad-padding decode can be caught as a plain ValueError.
_raised = False
try:
    base64.b64decode(b"abc")
except ValueError:
    _raised = True
assert _raised, "bad padding catchable as ValueError"
print("binascii_error_is_valueerror_subclass OK")
"###);
    assert_output(&out, r###"binascii_error_is_valueerror_subclass OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/base64/decoders_reject_non_ascii_str.py`.
#[test]
fn test_gen_errors_std_libs_base64_decoders_reject_non_ascii_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "decoders_reject_non_ascii_str"
# subject = "base64.b64decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b64decode: every str-accepting decoder (b64/standard_b64/urlsafe_b64/b32/b16/b85/a85) rejects a str containing non-ASCII characters with ValueError"""
import base64

_decoders = (
    ("b64decode", base64.b64decode),
    ("standard_b64decode", base64.standard_b64decode),
    ("urlsafe_b64decode", base64.urlsafe_b64decode),
    ("b32decode", base64.b32decode),
    ("b16decode", base64.b16decode),
    ("b85decode", base64.b85decode),
    ("a85decode", base64.a85decode),
)
for _name, _fn in _decoders:
    _raised = False
    try:
        _fn("with non-ascii Ë")
    except ValueError:
        _raised = True
    assert _raised, _name + " accepted non-ascii str"
print("decoders_reject_non_ascii_str OK")
"###);
    assert_output(&out, r###"decoders_reject_non_ascii_str OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/base64/legacy_encode_text_input_raises.py`.
#[test]
fn test_gen_errors_std_libs_base64_legacy_encode_text_input_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "legacy_encode_text_input_raises"
# subject = "base64.encode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.encode: legacy_encode_text_input_raises (errors)."""
import base64
from io import BytesIO, StringIO

_raised = False
try:
    base64.encode(StringIO('YWJj\n'), BytesIO())
except TypeError:
    _raised = True
assert _raised, "legacy_encode_text_input_raises: expected TypeError"
print("legacy_encode_text_input_raises OK")
"###);
    assert_output(&out, r###"legacy_encode_text_input_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/base64/urlsafe_b64decode_odd_padding_raises.py`.
#[test]
fn test_gen_errors_std_libs_base64_urlsafe_b64decode_odd_padding_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "urlsafe_b64decode_odd_padding_raises"
# subject = "base64.urlsafe_b64decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.urlsafe_b64decode: urlsafe_b64decode_odd_padding_raises (errors)."""
import base64
import binascii

_raised = False
try:
    base64.urlsafe_b64decode(b'abc')
except binascii.Error:
    _raised = True
assert _raised, "urlsafe_b64decode_odd_padding_raises: expected binascii.Error"
print("urlsafe_b64decode_odd_padding_raises OK")
"###);
    assert_output(&out, r###"urlsafe_b64decode_odd_padding_raises OK
"###);
}
