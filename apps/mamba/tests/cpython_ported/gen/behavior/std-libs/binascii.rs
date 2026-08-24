use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/binascii/a2b_base64_accepts_str.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_a2b_base64_accepts_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "a2b_base64_accepts_str"
# subject = "binascii.a2b_base64"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_base64: a2b_base64 accepts ASCII str input, not just bytes"""
import binascii

assert binascii.a2b_base64("aGVsbG8=") == b"hello", "a2b_base64 str input"
assert binascii.a2b_base64(b"aGVsbG8=") == b"hello", "a2b_base64 bytes input"

print("a2b_base64_accepts_str OK")
"###);
    assert_output(&out, r###"a2b_base64_accepts_str OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/a2b_base64_ignores_whitespace.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_a2b_base64_ignores_whitespace() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "a2b_base64_ignores_whitespace"
# subject = "binascii.a2b_base64"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_base64: a2b_base64 skips embedded whitespace/newlines while decoding"""
import binascii

assert binascii.a2b_base64(b"aGVs\nbG8=") == b"hello", "a2b with embedded newline"
assert binascii.a2b_base64(b"aGVsbG8=") == b"hello", "a2b without whitespace"

print("a2b_base64_ignores_whitespace OK")
"###);
    assert_output(&out, r###"a2b_base64_ignores_whitespace OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/a2b_qp_bad_escape_passthrough.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_a2b_qp_bad_escape_passthrough() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "a2b_qp_bad_escape_passthrough"
# subject = "binascii.a2b_qp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_qp: a2b_qp passes a malformed =escape through verbatim"""
import binascii

assert binascii.a2b_qp(b"=AX") == b"=AX", "bad escape passed through"

print("a2b_qp_bad_escape_passthrough OK")
"###);
    assert_output(&out, r###"a2b_qp_bad_escape_passthrough OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/a2b_qp_header_underscore.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_a2b_qp_header_underscore() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "a2b_qp_header_underscore"
# subject = "binascii.a2b_qp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_qp: a2b_qp leaves _ literal by default but maps it to space in header mode"""
import binascii

assert binascii.a2b_qp(b"_") == b"_", "underscore literal by default"
assert binascii.a2b_qp(b"_", header=True) == b" ", "underscore is space in header"

print("a2b_qp_header_underscore OK")
"###);
    assert_output(&out, r###"a2b_qp_header_underscore OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/a2b_qp_soft_break_and_escape.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_a2b_qp_soft_break_and_escape() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "a2b_qp_soft_break_and_escape"
# subject = "binascii.a2b_qp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_qp: a2b_qp: lone = is a soft break, == is literal =, =XX is case-insensitive hex"""
import binascii

# A lone '=' is a soft break and disappears; '==' decodes to a literal '='.
assert binascii.a2b_qp(b"=") == b"", "trailing = is soft break"
assert binascii.a2b_qp(b"==") == b"=", "== decodes to ="
# '=XX' is a hex escape (case-insensitive); a soft break joins the next line.
assert binascii.a2b_qp(b"=AB") == b"\xab", "=AB hex escape"
assert binascii.a2b_qp(b"=ab") == b"\xab", "lowercase hex escape"
assert binascii.a2b_qp(b"=\nAB") == b"AB", "soft break before text"

print("a2b_qp_soft_break_and_escape OK")
"###);
    assert_output(&out, r###"a2b_qp_soft_break_and_escape OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/a2b_uu_marker_length.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_a2b_uu_marker_length() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "a2b_uu_marker_length"
# subject = "binascii.a2b_uu"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_uu: a2b_uu derives output length from the leading marker byte; blank lines decode empty"""
import binascii

# a2b_uu derives output length only from the leading marker byte; trailing
# garbage is ignored, so high marker bytes yield runs of NUL.
assert binascii.a2b_uu(b"\x7f") == b"\x00" * 31, "marker 0x7f -> 31 NUL"
assert binascii.a2b_uu(b"\x80") == b"\x00" * 32, "marker 0x80 -> 32 NUL"
# Empty/blank lines decode to empty bytes.
assert binascii.a2b_uu(b" \n") == b"", "blank line decodes empty"
assert binascii.a2b_uu(b"`\n") == b"", "backtick line decodes empty"

print("a2b_uu_marker_length OK")
"###);
    assert_output(&out, r###"a2b_uu_marker_length OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/array_bin_ascii_test__test_unicode_b2a.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_array_bin_ascii_test__test_unicode_b2a() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "array_bin_ascii_test__test_unicode_b2a"
# subject = "cpython.test_binascii.ArrayBinASCIITest.test_unicode_b2a"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::ArrayBinASCIITest::test_unicode_b2a
"""Auto-ported test: ArrayBinASCIITest::test_unicode_b2a (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'

def type2test(s):
    return array.array('B', list(s))
self_data = type2test(rawdata)
for func in set(all_functions) - set(a2b_functions):
    try:

        try:
            getattr(binascii, func)('test')
            raise AssertionError('expected TypeError')
        except TypeError:
            pass
    except Exception as err:

        raise AssertionError('{}("test") raises {!r}'.format(func, err))

try:
    binascii.crc_hqx('test', 0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ArrayBinASCIITest::test_unicode_b2a: ok")
"###);
    assert_output(&out, r###"ArrayBinASCIITest::test_unicode_b2a: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/b2a_base64_newline_default.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_b2a_base64_newline_default() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "b2a_base64_newline_default"
# subject = "binascii.b2a_base64"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_base64: b2a_base64 appends a trailing newline by default (newline=True)"""
import binascii

_b64 = binascii.b2a_base64(b"abc")
assert isinstance(_b64, bytes), f"b2a_base64 type = {type(_b64)!r}"
assert _b64.endswith(b"\n"), f"b2a_base64 newline = {_b64!r}"
assert binascii.b2a_base64(b"abc", newline=True) == _b64, "newline=True is the default"
assert binascii.b2a_base64(b"hello") == b"aGVsbG8=\n", "known encoding with newline"

print("b2a_base64_newline_default OK")
"###);
    assert_output(&out, r###"b2a_base64_newline_default OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/b2a_base64_newline_false.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_b2a_base64_newline_false() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "b2a_base64_newline_false"
# subject = "binascii.b2a_base64"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_base64: b2a_base64(newline=False) drops the trailing newline"""
import binascii

_b64n = binascii.b2a_base64(b"abc", newline=False)
assert not _b64n.endswith(b"\n"), f"no newline = {_b64n!r}"
assert _b64n == b"YWJj", f"b2a_base64 no-newline = {_b64n!r}"

print("b2a_base64_newline_false OK")
"###);
    assert_output(&out, r###"b2a_base64_newline_false OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/b2a_qp_crlf_normalisation.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_b2a_qp_crlf_normalisation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "b2a_qp_crlf_normalisation"
# subject = "binascii.b2a_qp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_qp: b2a_qp normalises bare CR/LF to CRLF and escapes high bytes"""
import binascii

assert binascii.b2a_qp(b"\xff\r\n\xff\n\xff") == b"=FF\r\n=FF\r\n=FF", "CRLF + =FF"

print("b2a_qp_crlf_normalisation OK")
"###);
    assert_output(&out, r###"b2a_qp_crlf_normalisation OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/b2a_qp_escapes_reserved.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_b2a_qp_escapes_reserved() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "b2a_qp_escapes_reserved"
# subject = "binascii.b2a_qp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_qp: b2a_qp escapes non-printable/reserved bytes as =XX uppercase hex"""
import binascii

assert binascii.b2a_qp(b"\x7f") == b"=7F", "DEL escaped"
assert binascii.b2a_qp(b"=") == b"=3D", "literal = escaped"
assert binascii.b2a_qp(b" ") == b"=20", "lone trailing space escaped"
assert binascii.b2a_qp(b".") == b"=2E", "leading dot escaped"

print("b2a_qp_escapes_reserved OK")
"###);
    assert_output(&out, r###"b2a_qp_escapes_reserved OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/b2a_qp_header_and_quotetabs.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_b2a_qp_header_and_quotetabs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "b2a_qp_header_and_quotetabs"
# subject = "binascii.b2a_qp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_qp: b2a_qp header mode maps space to underscore; quotetabs escapes whitespace"""
import binascii

assert binascii.b2a_qp(b"_", header=True) == b"=5F", "underscore escaped in header"
assert binascii.b2a_qp(b"x y", header=True) == b"x_y", "space -> underscore"
assert binascii.b2a_qp(b"x y\tz", quotetabs=True) == b"x=20y=09z", "quotetabs escapes ws"

print("b2a_qp_header_and_quotetabs OK")
"###);
    assert_output(&out, r###"b2a_qp_header_and_quotetabs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/b2a_uu_backtick.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_b2a_uu_backtick() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "b2a_uu_backtick"
# subject = "binascii.b2a_uu"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_uu: b2a_uu(backtick=True) writes grave accents for zeros; both forms decode equal"""
import binascii

assert binascii.b2a_uu(b"", backtick=True) == b"`\n", "b2a_uu(empty, backtick)"
assert binascii.b2a_uu(b"\x00Cat", backtick=True) == b"$`$-A=```\n", "backtick encoding"
# Both space- and backtick-padded forms decode to the same bytes.
assert binascii.a2b_uu(b"$`$-A=```\n") == binascii.a2b_uu(b"$ $-A=   \n"), "backtick decode"

print("b2a_uu_backtick OK")
"###);
    assert_output(&out, r###"b2a_uu_backtick OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/b2a_uu_known_encodings.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_b2a_uu_known_encodings() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "b2a_uu_known_encodings"
# subject = "binascii.b2a_uu"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_uu: b2a_uu known encodings for single byte, empty, and a NUL-prefixed block"""
import binascii

assert binascii.b2a_uu(b"x") == b"!>   \n", "b2a_uu('x')"
assert binascii.b2a_uu(b"") == b" \n", "b2a_uu(empty)"
assert binascii.b2a_uu(b"\x00Cat") == b"$ $-A=   \n", "b2a_uu('\\x00Cat')"

print("b2a_uu_known_encodings OK")
"###);
    assert_output(&out, r###"b2a_uu_known_encodings OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/base64_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_base64_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "base64_roundtrip"
# subject = "binascii.a2b_base64"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_base64: a2b_base64 inverts b2a_base64 over arbitrary bytes (round-trip)"""
import binascii

_data = bytes(range(0, 256, 3))
_encoded = binascii.b2a_base64(_data, newline=False)
assert binascii.a2b_base64(_encoded) == _data, "base64 round-trip"

print("base64_roundtrip OK")
"###);
    assert_output(&out, r###"base64_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/bin_ascii_test__test_b2a_base64_newline.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_bin_ascii_test__test_b2a_base64_newline() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bin_ascii_test__test_b2a_base64_newline"
# subject = "cpython.test_binascii.BinASCIITest.test_b2a_base64_newline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BinASCIITest::test_b2a_base64_newline
"""Auto-ported test: BinASCIITest::test_b2a_base64_newline (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
self_data = type2test(rawdata)
b = type2test(b'hello')

assert binascii.b2a_base64(b) == b'aGVsbG8=\n'

assert binascii.b2a_base64(b, newline=True) == b'aGVsbG8=\n'

assert binascii.b2a_base64(b, newline=False) == b'aGVsbG8='
print("BinASCIITest::test_b2a_base64_newline: ok")
"###);
    assert_output(&out, r###"BinASCIITest::test_b2a_base64_newline: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/bin_ascii_test__test_base64errors.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_bin_ascii_test__test_base64errors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bin_ascii_test__test_base64errors"
# subject = "cpython.test_binascii.BinASCIITest.test_base64errors"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BinASCIITest::test_base64errors
"""Auto-ported test: BinASCIITest::test_base64errors (CPython 3.12 oracle)."""


import binascii
import re


def assert_raises_regex(exc_type, pattern, fn, *args, **kwargs):
    try:
        fn(*args, **kwargs)
    except exc_type as exc:
        assert re.search(pattern, str(exc)), (pattern, str(exc))
        return
    raise AssertionError(f"expected {exc_type.__name__}")


def assert_incorrect_padding(data):
    assert_raises_regex(
        binascii.Error,
        r"(?i)Incorrect padding",
        binascii.a2b_base64,
        bytes(data),
    )


assert_incorrect_padding(b"ab")
assert_incorrect_padding(b"ab=")
assert_incorrect_padding(b"abc")
assert_incorrect_padding(b"abcdef")
assert_incorrect_padding(b"abcdef=")
assert_incorrect_padding(b"abcdefg")
assert_incorrect_padding(b"a=b=")
assert_incorrect_padding(b"a\nb=")


def assert_invalid_length(data):
    n_data_chars = len(re.sub(br"[^A-Za-z0-9/+]", b"", data))
    expected_errmsg_re = r"(?i)Invalid.+number of data characters.+" + str(
        n_data_chars
    )
    assert_raises_regex(
        binascii.Error,
        expected_errmsg_re,
        binascii.a2b_base64,
        bytes(data),
    )


assert_invalid_length(b"a")
assert_invalid_length(b"a=")
assert_invalid_length(b"a==")
assert_invalid_length(b"a===")
assert_invalid_length(b"a" * 5)
assert_invalid_length(b"a" * (4 * 87 + 1))
assert_invalid_length(b"A\tB\nC ??DE")

print("BinASCIITest::test_base64errors: ok")
"###);
    assert_output(&out, r###"BinASCIITest::test_base64errors: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/bin_ascii_test__test_base64invalid.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_bin_ascii_test__test_base64invalid() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bin_ascii_test__test_base64invalid"
# subject = "cpython.test_binascii.BinASCIITest.test_base64invalid"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BinASCIITest::test_base64invalid
"""Auto-ported test: BinASCIITest::test_base64invalid (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
self_data = type2test(rawdata)
MAX_BASE64 = 57
lines = []
for i in range(0, len(self_data), MAX_BASE64):
    b = type2test(rawdata[i:i + MAX_BASE64])
    a = binascii.b2a_base64(b)
    lines.append(a)
fillers = bytearray()
valid = b'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/'
for i in range(256):
    if i not in valid:
        fillers.append(i)

def addnoise(line):
    noise = fillers
    ratio = len(line) // len(noise)
    res = bytearray()
    while line and noise:
        if len(line) // len(noise) > ratio:
            c, line = (line[0], line[1:])
        else:
            c, noise = (noise[0], noise[1:])
        res.append(c)
    return res + noise + line
res = bytearray()
for line in map(addnoise, lines):
    a = type2test(line)
    b = binascii.a2b_base64(a)
    res += b

assert res == rawdata

assert binascii.a2b_base64(type2test(fillers)) == b''
print("BinASCIITest::test_base64invalid: ok")
"###);
    assert_output(&out, r###"BinASCIITest::test_base64invalid: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/bin_ascii_test__test_base64valid.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_bin_ascii_test__test_base64valid() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bin_ascii_test__test_base64valid"
# subject = "cpython.test_binascii.BinASCIITest.test_base64valid"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BinASCIITest::test_base64valid
"""Auto-ported test: BinASCIITest::test_base64valid (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
self_data = type2test(rawdata)
MAX_BASE64 = 57
lines = []
for i in range(0, len(rawdata), MAX_BASE64):
    b = type2test(rawdata[i:i + MAX_BASE64])
    a = binascii.b2a_base64(b)
    lines.append(a)
res = bytes()
for line in lines:
    a = type2test(line)
    b = binascii.a2b_base64(a)
    res += b

assert res == rawdata
print("BinASCIITest::test_base64valid: ok")
"###);
    assert_output(&out, r###"BinASCIITest::test_base64valid: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/bin_ascii_test__test_empty_string.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_bin_ascii_test__test_empty_string() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bin_ascii_test__test_empty_string"
# subject = "cpython.test_binascii.BinASCIITest.test_empty_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BinASCIITest::test_empty_string
"""Auto-ported test: BinASCIITest::test_empty_string (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
self_data = type2test(rawdata)
empty = type2test(b'')
for func in all_functions:
    if func == 'crc_hqx':
        binascii.crc_hqx(empty, 0)
        continue
    f = getattr(binascii, func)
    try:
        f(empty)
    except Exception as err:

        raise AssertionError('{}({!r}) raises {!r}'.format(func, empty, err))
print("BinASCIITest::test_empty_string: ok")
"###);
    assert_output(&out, r###"BinASCIITest::test_empty_string: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/bin_ascii_test__test_hex.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_bin_ascii_test__test_hex() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bin_ascii_test__test_hex"
# subject = "cpython.test_binascii.BinASCIITest.test_hex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BinASCIITest::test_hex
"""Auto-ported test: BinASCIITest::test_hex (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
self_data = type2test(rawdata)
s = b'{s\x05\x00\x00\x00worldi\x02\x00\x00\x00s\x05\x00\x00\x00helloi\x01\x00\x00\x000'
t = binascii.b2a_hex(type2test(s))
u = binascii.a2b_hex(type2test(t))

assert s == u

try:
    binascii.a2b_hex(t[:-1])
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(t[:-1] + b'q')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(bytes([255, 255]))
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(b'0G')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(b'0g')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(b'G0')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(b'g0')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

assert binascii.hexlify(type2test(s)) == t

assert binascii.unhexlify(type2test(t)) == u
print("BinASCIITest::test_hex: ok")
"###);
    assert_output(&out, r###"BinASCIITest::test_hex: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/bin_ascii_test__test_hex_separator.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_bin_ascii_test__test_hex_separator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bin_ascii_test__test_hex_separator"
# subject = "cpython.test_binascii.BinASCIITest.test_hex_separator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BinASCIITest::test_hex_separator
"""Auto-ported test: BinASCIITest::test_hex_separator (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
self_data = type2test(rawdata)
'Test that hexlify and b2a_hex are binary versions of bytes.hex.'
s = b'{s\x05\x00\x00\x00worldi\x02\x00\x00\x00s\x05\x00\x00\x00helloi\x01\x00\x00\x000'

assert binascii.hexlify(type2test(s)) == s.hex().encode('ascii')
expected8 = s.hex('.', 8).encode('ascii')

assert binascii.hexlify(type2test(s), '.', 8) == expected8
expected1 = s.hex(':').encode('ascii')

assert binascii.b2a_hex(type2test(s), ':') == expected1
print("BinASCIITest::test_hex_separator: ok")
"###);
    assert_output(&out, r###"BinASCIITest::test_hex_separator: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/bin_ascii_test__test_unicode_a2b.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_bin_ascii_test__test_unicode_a2b() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bin_ascii_test__test_unicode_a2b"
# subject = "cpython.test_binascii.BinASCIITest.test_unicode_a2b"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BinASCIITest::test_unicode_a2b
"""Auto-ported test: BinASCIITest::test_unicode_a2b (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
self_data = type2test(rawdata)
MAX_ALL = 45
raw = rawdata[:MAX_ALL]
for fa, fb in zip(a2b_functions, b2a_functions):
    a2b = getattr(binascii, fa)
    b2a = getattr(binascii, fb)
    try:
        a = b2a(type2test(raw))
        binary_res = a2b(a)
        a = a.decode('ascii')
        res = a2b(a)
    except Exception as err:

        raise AssertionError('{}/{} conversion raises {!r}'.format(fb, fa, err))

    assert res == raw

    assert res == binary_res

    assert isinstance(res, bytes)

    try:
        a2b('\x80')
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
print("BinASCIITest::test_unicode_a2b: ok")
"###);
    assert_output(&out, r###"BinASCIITest::test_unicode_a2b: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/bin_ascii_test__test_unicode_b2a.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_bin_ascii_test__test_unicode_b2a() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bin_ascii_test__test_unicode_b2a"
# subject = "cpython.test_binascii.BinASCIITest.test_unicode_b2a"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BinASCIITest::test_unicode_b2a
"""Auto-ported test: BinASCIITest::test_unicode_b2a (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
self_data = type2test(rawdata)
for func in set(all_functions) - set(a2b_functions):
    try:

        try:
            getattr(binascii, func)('test')
            raise AssertionError('expected TypeError')
        except TypeError:
            pass
    except Exception as err:

        raise AssertionError('{}("test") raises {!r}'.format(func, err))

try:
    binascii.crc_hqx('test', 0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("BinASCIITest::test_unicode_b2a: ok")
"###);
    assert_output(&out, r###"BinASCIITest::test_unicode_b2a: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/bytearray_bin_ascii_test__test_b2a_base64_newline.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_bytearray_bin_ascii_test__test_b2a_base64_newline() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bytearray_bin_ascii_test__test_b2a_base64_newline"
# subject = "cpython.test_binascii.BytearrayBinASCIITest.test_b2a_base64_newline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BytearrayBinASCIITest::test_b2a_base64_newline
"""Auto-ported test: BytearrayBinASCIITest::test_b2a_base64_newline (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
type2test = bytearray
self_data = type2test(rawdata)
b = type2test(b'hello')

assert binascii.b2a_base64(b) == b'aGVsbG8=\n'

assert binascii.b2a_base64(b, newline=True) == b'aGVsbG8=\n'

assert binascii.b2a_base64(b, newline=False) == b'aGVsbG8='
print("BytearrayBinASCIITest::test_b2a_base64_newline: ok")
"###);
    assert_output(&out, r###"BytearrayBinASCIITest::test_b2a_base64_newline: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/bytearray_bin_ascii_test__test_base64invalid.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_bytearray_bin_ascii_test__test_base64invalid() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bytearray_bin_ascii_test__test_base64invalid"
# subject = "cpython.test_binascii.BytearrayBinASCIITest.test_base64invalid"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BytearrayBinASCIITest::test_base64invalid
"""Auto-ported test: BytearrayBinASCIITest::test_base64invalid (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
type2test = bytearray
self_data = type2test(rawdata)
MAX_BASE64 = 57
lines = []
for i in range(0, len(self_data), MAX_BASE64):
    b = type2test(rawdata[i:i + MAX_BASE64])
    a = binascii.b2a_base64(b)
    lines.append(a)
fillers = bytearray()
valid = b'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/'
for i in range(256):
    if i not in valid:
        fillers.append(i)

def addnoise(line):
    noise = fillers
    ratio = len(line) // len(noise)
    res = bytearray()
    while line and noise:
        if len(line) // len(noise) > ratio:
            c, line = (line[0], line[1:])
        else:
            c, noise = (noise[0], noise[1:])
        res.append(c)
    return res + noise + line
res = bytearray()
for line in map(addnoise, lines):
    a = type2test(line)
    b = binascii.a2b_base64(a)
    res += b

assert res == rawdata

assert binascii.a2b_base64(type2test(fillers)) == b''
print("BytearrayBinASCIITest::test_base64invalid: ok")
"###);
    assert_output(&out, r###"BytearrayBinASCIITest::test_base64invalid: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/bytearray_bin_ascii_test__test_base64valid.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_bytearray_bin_ascii_test__test_base64valid() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bytearray_bin_ascii_test__test_base64valid"
# subject = "cpython.test_binascii.BytearrayBinASCIITest.test_base64valid"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BytearrayBinASCIITest::test_base64valid
"""Auto-ported test: BytearrayBinASCIITest::test_base64valid (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
type2test = bytearray
self_data = type2test(rawdata)
MAX_BASE64 = 57
lines = []
for i in range(0, len(rawdata), MAX_BASE64):
    b = type2test(rawdata[i:i + MAX_BASE64])
    a = binascii.b2a_base64(b)
    lines.append(a)
res = bytes()
for line in lines:
    a = type2test(line)
    b = binascii.a2b_base64(a)
    res += b

assert res == rawdata
print("BytearrayBinASCIITest::test_base64valid: ok")
"###);
    assert_output(&out, r###"BytearrayBinASCIITest::test_base64valid: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/bytearray_bin_ascii_test__test_empty_string.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_bytearray_bin_ascii_test__test_empty_string() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bytearray_bin_ascii_test__test_empty_string"
# subject = "cpython.test_binascii.BytearrayBinASCIITest.test_empty_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BytearrayBinASCIITest::test_empty_string
"""Auto-ported test: BytearrayBinASCIITest::test_empty_string (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
type2test = bytearray
self_data = type2test(rawdata)
empty = type2test(b'')
for func in all_functions:
    if func == 'crc_hqx':
        binascii.crc_hqx(empty, 0)
        continue
    f = getattr(binascii, func)
    try:
        f(empty)
    except Exception as err:

        raise AssertionError('{}({!r}) raises {!r}'.format(func, empty, err))
print("BytearrayBinASCIITest::test_empty_string: ok")
"###);
    assert_output(&out, r###"BytearrayBinASCIITest::test_empty_string: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/bytearray_bin_ascii_test__test_hex.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_bytearray_bin_ascii_test__test_hex() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bytearray_bin_ascii_test__test_hex"
# subject = "cpython.test_binascii.BytearrayBinASCIITest.test_hex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BytearrayBinASCIITest::test_hex
"""Auto-ported test: BytearrayBinASCIITest::test_hex (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
type2test = bytearray
self_data = type2test(rawdata)
s = b'{s\x05\x00\x00\x00worldi\x02\x00\x00\x00s\x05\x00\x00\x00helloi\x01\x00\x00\x000'
t = binascii.b2a_hex(type2test(s))
u = binascii.a2b_hex(type2test(t))

assert s == u

try:
    binascii.a2b_hex(t[:-1])
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(t[:-1] + b'q')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(bytes([255, 255]))
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(b'0G')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(b'0g')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(b'G0')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(b'g0')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

assert binascii.hexlify(type2test(s)) == t

assert binascii.unhexlify(type2test(t)) == u
print("BytearrayBinASCIITest::test_hex: ok")
"###);
    assert_output(&out, r###"BytearrayBinASCIITest::test_hex: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/bytearray_bin_ascii_test__test_hex_separator.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_bytearray_bin_ascii_test__test_hex_separator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bytearray_bin_ascii_test__test_hex_separator"
# subject = "cpython.test_binascii.BytearrayBinASCIITest.test_hex_separator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BytearrayBinASCIITest::test_hex_separator
"""Auto-ported test: BytearrayBinASCIITest::test_hex_separator (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
type2test = bytearray
self_data = type2test(rawdata)
'Test that hexlify and b2a_hex are binary versions of bytes.hex.'
s = b'{s\x05\x00\x00\x00worldi\x02\x00\x00\x00s\x05\x00\x00\x00helloi\x01\x00\x00\x000'

assert binascii.hexlify(type2test(s)) == s.hex().encode('ascii')
expected8 = s.hex('.', 8).encode('ascii')

assert binascii.hexlify(type2test(s), '.', 8) == expected8
expected1 = s.hex(':').encode('ascii')

assert binascii.b2a_hex(type2test(s), ':') == expected1
print("BytearrayBinASCIITest::test_hex_separator: ok")
"###);
    assert_output(&out, r###"BytearrayBinASCIITest::test_hex_separator: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/bytearray_bin_ascii_test__test_unicode_a2b.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_bytearray_bin_ascii_test__test_unicode_a2b() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bytearray_bin_ascii_test__test_unicode_a2b"
# subject = "cpython.test_binascii.BytearrayBinASCIITest.test_unicode_a2b"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BytearrayBinASCIITest::test_unicode_a2b
"""Auto-ported test: BytearrayBinASCIITest::test_unicode_a2b (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
type2test = bytearray
self_data = type2test(rawdata)
MAX_ALL = 45
raw = rawdata[:MAX_ALL]
for fa, fb in zip(a2b_functions, b2a_functions):
    a2b = getattr(binascii, fa)
    b2a = getattr(binascii, fb)
    try:
        a = b2a(type2test(raw))
        binary_res = a2b(a)
        a = a.decode('ascii')
        res = a2b(a)
    except Exception as err:

        raise AssertionError('{}/{} conversion raises {!r}'.format(fb, fa, err))

    assert res == raw

    assert res == binary_res

    assert isinstance(res, bytes)

    try:
        a2b('\x80')
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
print("BytearrayBinASCIITest::test_unicode_a2b: ok")
"###);
    assert_output(&out, r###"BytearrayBinASCIITest::test_unicode_a2b: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/bytearray_bin_ascii_test__test_unicode_b2a.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_bytearray_bin_ascii_test__test_unicode_b2a() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bytearray_bin_ascii_test__test_unicode_b2a"
# subject = "cpython.test_binascii.BytearrayBinASCIITest.test_unicode_b2a"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BytearrayBinASCIITest::test_unicode_b2a
"""Auto-ported test: BytearrayBinASCIITest::test_unicode_b2a (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
type2test = bytearray
self_data = type2test(rawdata)
for func in set(all_functions) - set(a2b_functions):
    try:

        try:
            getattr(binascii, func)('test')
            raise AssertionError('expected TypeError')
        except TypeError:
            pass
    except Exception as err:

        raise AssertionError('{}("test") raises {!r}'.format(func, err))

try:
    binascii.crc_hqx('test', 0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("BytearrayBinASCIITest::test_unicode_b2a: ok")
"###);
    assert_output(&out, r###"BytearrayBinASCIITest::test_unicode_b2a: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/codecs_accept_empty_input.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_codecs_accept_empty_input() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "codecs_accept_empty_input"
# subject = "binascii.hexlify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.hexlify: every b2a_*/a2b_* codec accepts empty input without raising"""
import binascii

for _name in ("b2a_base64", "b2a_hex", "b2a_qp", "b2a_uu", "hexlify",
              "a2b_base64", "a2b_hex", "a2b_qp", "a2b_uu", "unhexlify"):
    getattr(binascii, _name)(b"")
assert binascii.crc_hqx(b"", 0) == 0, "crc_hqx empty"
assert binascii.crc32(b"") == 0, "crc32 empty"

print("codecs_accept_empty_input OK")
"###);
    assert_output(&out, r###"codecs_accept_empty_input OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/crc32_known_and_incremental.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_crc32_known_and_incremental() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "crc32_known_and_incremental"
# subject = "binascii.crc32"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.crc32: crc32 known values plus incremental seeding equals one-shot"""
import binascii

_crc = binascii.crc32(b"test data")
assert isinstance(_crc, int), f"crc32 type = {type(_crc)!r}"
assert binascii.crc32(b"") == 0, "crc32(empty) = 0"
assert binascii.crc32(b"hello") == 907060870, "crc32(hello)"
_crc1 = binascii.crc32(b"hel")
assert binascii.crc32(b"lo", _crc1) == binascii.crc32(b"hello"), "incremental crc32"

print("crc32_known_and_incremental OK")
"###);
    assert_output(&out, r###"crc32_known_and_incremental OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/crc_hqx_known_and_mask.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_crc_hqx_known_and_mask() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "crc_hqx_known_and_mask"
# subject = "binascii.crc_hqx"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.crc_hqx: crc_hqx incremental value, and empty-data returns seed masked to 16 bits"""
import binascii

_q = binascii.crc_hqx(b"Test the CRC-32 of", 0)
assert isinstance(_q, int), f"crc_hqx type = {type(_q)!r}"
_q = binascii.crc_hqx(b" this string.", _q)
assert _q == 14290, f"crc_hqx incremental = {_q}"
# Empty data returns the seed masked to 16 bits.
assert binascii.crc_hqx(b"", 0) == 0, "crc_hqx empty seed 0"
assert binascii.crc_hqx(b"", -1) == 0xffff, "crc_hqx masks seed to 16 bits"
assert binascii.crc_hqx(b"", 0x12345678) == 0x5678, "crc_hqx 16-bit mask"

print("crc_hqx_known_and_mask OK")
"###);
    assert_output(&out, r###"crc_hqx_known_and_mask OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/hex_aliases_match.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_hex_aliases_match() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "hex_aliases_match"
# subject = "binascii.b2a_hex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_hex: b2a_hex aliases hexlify and a2b_hex aliases unhexlify"""
import binascii

_data = b"hello world"
assert binascii.b2a_hex(_data) == binascii.hexlify(_data), "b2a_hex == hexlify"
assert binascii.a2b_hex(b"0102ff") == binascii.unhexlify(b"0102ff"), "a2b_hex == unhexlify"

print("hex_aliases_match OK")
"###);
    assert_output(&out, r###"hex_aliases_match OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/hex_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_hex_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "hex_roundtrip"
# subject = "binascii.unhexlify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.unhexlify: unhexlify inverts hexlify over arbitrary bytes (round-trip)"""
import binascii

_data = b"hello world"
_hex = binascii.hexlify(_data)
assert isinstance(_hex, bytes), f"hexlify type = {type(_hex)!r}"
_raw = binascii.unhexlify(_hex)
assert isinstance(_raw, bytes), f"unhexlify type = {type(_raw)!r}"
assert _raw == _data, f"hex round-trip = {_raw!r}"
assert binascii.unhexlify(b"0102ff") == b"\x01\x02\xff", "unhexlify 0102ff"

print("hex_roundtrip OK")
"###);
    assert_output(&out, r###"hex_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/hexlify_lowercase_hex.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_hexlify_lowercase_hex() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "hexlify_lowercase_hex"
# subject = "binascii.hexlify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.hexlify: hexlify emits lowercase hex digits and is bytes-typed"""
import binascii

_h = binascii.hexlify(b"\xab\xcd\xef")
assert isinstance(_h, bytes), f"hexlify type = {type(_h)!r}"
assert _h == b"abcdef", f"lowercase hex = {_h!r}"
print("hexlify_lowercase_hex OK")
"###);
    assert_output(&out, r###"hexlify_lowercase_hex OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/hexlify_separator_and_group.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_hexlify_separator_and_group() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "hexlify_separator_and_group"
# subject = "binascii.hexlify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.hexlify: hexlify(sep) and hexlify(sep, bytes_per_group) match bytes.hex"""
import binascii

# A bare separator inserts one between every byte.
_hex_sep = binascii.hexlify(b"\x01\x02\x03", ":")
assert isinstance(_hex_sep, bytes), f"hexlify sep type = {type(_hex_sep)!r}"
assert _hex_sep == b"01:02:03", f"hexlify with sep = {_hex_sep!r}"

# A separator plus a bytes-per-group count groups from the right.
_payload = bytes(range(1, 11))
_grouped = binascii.hexlify(_payload, ".", 4)
assert _grouped == b"0102.03040506.0708090a", f"grouped hex = {_grouped!r}"
assert _grouped == _payload.hex(".", 4).encode("ascii"), "matches bytes.hex"

print("hexlify_separator_and_group OK")
"###);
    assert_output(&out, r###"hexlify_separator_and_group OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/memoryview_bin_ascii_test__test_b2a_base64_newline.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_memoryview_bin_ascii_test__test_b2a_base64_newline() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "memoryview_bin_ascii_test__test_b2a_base64_newline"
# subject = "cpython.test_binascii.MemoryviewBinASCIITest.test_b2a_base64_newline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::MemoryviewBinASCIITest::test_b2a_base64_newline
"""Auto-ported test: MemoryviewBinASCIITest::test_b2a_base64_newline (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
type2test = memoryview
self_data = type2test(rawdata)
b = type2test(b'hello')

assert binascii.b2a_base64(b) == b'aGVsbG8=\n'

assert binascii.b2a_base64(b, newline=True) == b'aGVsbG8=\n'

assert binascii.b2a_base64(b, newline=False) == b'aGVsbG8='
print("MemoryviewBinASCIITest::test_b2a_base64_newline: ok")
"###);
    assert_output(&out, r###"MemoryviewBinASCIITest::test_b2a_base64_newline: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/memoryview_bin_ascii_test__test_base64invalid.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_memoryview_bin_ascii_test__test_base64invalid() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "memoryview_bin_ascii_test__test_base64invalid"
# subject = "cpython.test_binascii.MemoryviewBinASCIITest.test_base64invalid"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::MemoryviewBinASCIITest::test_base64invalid
"""Auto-ported test: MemoryviewBinASCIITest::test_base64invalid (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
type2test = memoryview
self_data = type2test(rawdata)
MAX_BASE64 = 57
lines = []
for i in range(0, len(self_data), MAX_BASE64):
    b = type2test(rawdata[i:i + MAX_BASE64])
    a = binascii.b2a_base64(b)
    lines.append(a)
fillers = bytearray()
valid = b'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/'
for i in range(256):
    if i not in valid:
        fillers.append(i)

def addnoise(line):
    noise = fillers
    ratio = len(line) // len(noise)
    res = bytearray()
    while line and noise:
        if len(line) // len(noise) > ratio:
            c, line = (line[0], line[1:])
        else:
            c, noise = (noise[0], noise[1:])
        res.append(c)
    return res + noise + line
res = bytearray()
for line in map(addnoise, lines):
    a = type2test(line)
    b = binascii.a2b_base64(a)
    res += b

assert res == rawdata

assert binascii.a2b_base64(type2test(fillers)) == b''
print("MemoryviewBinASCIITest::test_base64invalid: ok")
"###);
    assert_output(&out, r###"MemoryviewBinASCIITest::test_base64invalid: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/memoryview_bin_ascii_test__test_base64valid.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_memoryview_bin_ascii_test__test_base64valid() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "memoryview_bin_ascii_test__test_base64valid"
# subject = "cpython.test_binascii.MemoryviewBinASCIITest.test_base64valid"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::MemoryviewBinASCIITest::test_base64valid
"""Auto-ported test: MemoryviewBinASCIITest::test_base64valid (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
type2test = memoryview
self_data = type2test(rawdata)
MAX_BASE64 = 57
lines = []
for i in range(0, len(rawdata), MAX_BASE64):
    b = type2test(rawdata[i:i + MAX_BASE64])
    a = binascii.b2a_base64(b)
    lines.append(a)
res = bytes()
for line in lines:
    a = type2test(line)
    b = binascii.a2b_base64(a)
    res += b

assert res == rawdata
print("MemoryviewBinASCIITest::test_base64valid: ok")
"###);
    assert_output(&out, r###"MemoryviewBinASCIITest::test_base64valid: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/memoryview_bin_ascii_test__test_empty_string.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_memoryview_bin_ascii_test__test_empty_string() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "memoryview_bin_ascii_test__test_empty_string"
# subject = "cpython.test_binascii.MemoryviewBinASCIITest.test_empty_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::MemoryviewBinASCIITest::test_empty_string
"""Auto-ported test: MemoryviewBinASCIITest::test_empty_string (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
type2test = memoryview
self_data = type2test(rawdata)
empty = type2test(b'')
for func in all_functions:
    if func == 'crc_hqx':
        binascii.crc_hqx(empty, 0)
        continue
    f = getattr(binascii, func)
    try:
        f(empty)
    except Exception as err:

        raise AssertionError('{}({!r}) raises {!r}'.format(func, empty, err))
print("MemoryviewBinASCIITest::test_empty_string: ok")
"###);
    assert_output(&out, r###"MemoryviewBinASCIITest::test_empty_string: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/memoryview_bin_ascii_test__test_hex.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_memoryview_bin_ascii_test__test_hex() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "memoryview_bin_ascii_test__test_hex"
# subject = "cpython.test_binascii.MemoryviewBinASCIITest.test_hex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::MemoryviewBinASCIITest::test_hex
"""Auto-ported test: MemoryviewBinASCIITest::test_hex (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
type2test = memoryview
self_data = type2test(rawdata)
s = b'{s\x05\x00\x00\x00worldi\x02\x00\x00\x00s\x05\x00\x00\x00helloi\x01\x00\x00\x000'
t = binascii.b2a_hex(type2test(s))
u = binascii.a2b_hex(type2test(t))

assert s == u

try:
    binascii.a2b_hex(t[:-1])
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(t[:-1] + b'q')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(bytes([255, 255]))
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(b'0G')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(b'0g')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(b'G0')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(b'g0')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

assert binascii.hexlify(type2test(s)) == t

assert binascii.unhexlify(type2test(t)) == u
print("MemoryviewBinASCIITest::test_hex: ok")
"###);
    assert_output(&out, r###"MemoryviewBinASCIITest::test_hex: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/memoryview_bin_ascii_test__test_hex_separator.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_memoryview_bin_ascii_test__test_hex_separator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "memoryview_bin_ascii_test__test_hex_separator"
# subject = "cpython.test_binascii.MemoryviewBinASCIITest.test_hex_separator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::MemoryviewBinASCIITest::test_hex_separator
"""Auto-ported test: MemoryviewBinASCIITest::test_hex_separator (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
type2test = memoryview
self_data = type2test(rawdata)
'Test that hexlify and b2a_hex are binary versions of bytes.hex.'
s = b'{s\x05\x00\x00\x00worldi\x02\x00\x00\x00s\x05\x00\x00\x00helloi\x01\x00\x00\x000'

assert binascii.hexlify(type2test(s)) == s.hex().encode('ascii')
expected8 = s.hex('.', 8).encode('ascii')

assert binascii.hexlify(type2test(s), '.', 8) == expected8
expected1 = s.hex(':').encode('ascii')

assert binascii.b2a_hex(type2test(s), ':') == expected1
print("MemoryviewBinASCIITest::test_hex_separator: ok")
"###);
    assert_output(&out, r###"MemoryviewBinASCIITest::test_hex_separator: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/memoryview_bin_ascii_test__test_unicode_a2b.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_memoryview_bin_ascii_test__test_unicode_a2b() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "memoryview_bin_ascii_test__test_unicode_a2b"
# subject = "cpython.test_binascii.MemoryviewBinASCIITest.test_unicode_a2b"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::MemoryviewBinASCIITest::test_unicode_a2b
"""Auto-ported test: MemoryviewBinASCIITest::test_unicode_a2b (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
type2test = memoryview
self_data = type2test(rawdata)
MAX_ALL = 45
raw = rawdata[:MAX_ALL]
for fa, fb in zip(a2b_functions, b2a_functions):
    a2b = getattr(binascii, fa)
    b2a = getattr(binascii, fb)
    try:
        a = b2a(type2test(raw))
        binary_res = a2b(a)
        a = a.decode('ascii')
        res = a2b(a)
    except Exception as err:

        raise AssertionError('{}/{} conversion raises {!r}'.format(fb, fa, err))

    assert res == raw

    assert res == binary_res

    assert isinstance(res, bytes)

    try:
        a2b('\x80')
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
print("MemoryviewBinASCIITest::test_unicode_a2b: ok")
"###);
    assert_output(&out, r###"MemoryviewBinASCIITest::test_unicode_a2b: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/memoryview_bin_ascii_test__test_unicode_b2a.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_memoryview_bin_ascii_test__test_unicode_b2a() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "memoryview_bin_ascii_test__test_unicode_b2a"
# subject = "cpython.test_binascii.MemoryviewBinASCIITest.test_unicode_b2a"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::MemoryviewBinASCIITest::test_unicode_b2a
"""Auto-ported test: MemoryviewBinASCIITest::test_unicode_b2a (CPython 3.12 oracle)."""


import unittest
import binascii
import array
import re
from test.support import bigmemtest, _1G, _4G


'Test the binascii C module.'

b2a_functions = ['b2a_base64', 'b2a_hex', 'b2a_qp', 'b2a_uu', 'hexlify']

a2b_functions = ['a2b_base64', 'a2b_hex', 'a2b_qp', 'a2b_uu', 'unhexlify']

all_functions = a2b_functions + b2a_functions + ['crc32', 'crc_hqx']


# --- test body ---
type2test = bytes
rawdata = b'The quick brown fox jumps over the lazy dog.\r\n'
type2test = memoryview
self_data = type2test(rawdata)
for func in set(all_functions) - set(a2b_functions):
    try:

        try:
            getattr(binascii, func)('test')
            raise AssertionError('expected TypeError')
        except TypeError:
            pass
    except Exception as err:

        raise AssertionError('{}("test") raises {!r}'.format(func, err))

try:
    binascii.crc_hqx('test', 0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("MemoryviewBinASCIITest::test_unicode_b2a: ok")
"###);
    assert_output(&out, r###"MemoryviewBinASCIITest::test_unicode_b2a: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/unhexlify_accepts_str.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_unhexlify_accepts_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "unhexlify_accepts_str"
# subject = "binascii.unhexlify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.unhexlify: unhexlify and a2b_hex accept ASCII str input, not just bytes"""
import binascii

assert binascii.unhexlify("deadbeef") == b"\xde\xad\xbe\xef", "str unhexlify"
assert binascii.a2b_hex("0102ff") == b"\x01\x02\xff", "a2b_hex str input"

print("unhexlify_accepts_str OK")
"###);
    assert_output(&out, r###"unhexlify_accepts_str OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/binascii/uu_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_binascii_uu_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "uu_roundtrip"
# subject = "binascii.b2a_uu"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_uu: a2b_uu inverts b2a_uu over a 45-byte block (per-line maximum)"""
import binascii

_raw = b"The quick brown fox jumps over the lazy dog.."[:45]
_line = binascii.b2a_uu(_raw)
assert binascii.a2b_uu(_line) == _raw, f"uu round-trip = {_line!r}"

print("uu_roundtrip OK")
"###);
    assert_output(&out, r###"uu_roundtrip OK
"###);
}
