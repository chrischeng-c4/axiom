use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/base64/a85_adobe_framing_round_trip.py`.
#[test]
fn test_gen_behavior_std_libs_base64_a85_adobe_framing_round_trip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "a85_adobe_framing_round_trip"
# subject = "base64.a85encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.a85encode: a85encode(adobe=True) wraps output in '<~'/'~>' framing and a85decode(adobe=True) round-trips it, including the empty stream '<~~>'"""
import base64

_adobe = base64.a85encode(b"hello", adobe=True)
assert _adobe.startswith(b"<~") and _adobe.endswith(b"~>"), _adobe
assert base64.a85decode(_adobe, adobe=True) == b"hello", "adobe round-trip"
# An empty Adobe stream decodes to empty bytes.
assert base64.a85decode(b"<~~>", adobe=True) == b"", "adobe empty"
print("a85_adobe_framing_round_trip OK")
"###);
    assert_output(&out, r###"a85_adobe_framing_round_trip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/a85_all_bytes_round_trip.py`.
#[test]
fn test_gen_behavior_std_libs_base64_a85_all_bytes_round_trip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "a85_all_bytes_round_trip"
# subject = "base64.a85encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.a85encode: a85encode then a85decode round-trips every byte value 0..255 unchanged"""
import base64

_payload = bytes(range(256))
_enc = base64.a85encode(_payload)
assert isinstance(_enc, bytes), type(_enc)
assert base64.a85decode(_enc) == _payload, "a85 round-trip"
print("a85_all_bytes_round_trip OK")
"###);
    assert_output(&out, r###"a85_all_bytes_round_trip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/b16_rfc4648_vectors.py`.
#[test]
fn test_gen_behavior_std_libs_base64_b16_rfc4648_vectors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b16_rfc4648_vectors"
# subject = "base64.b16encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b16encode: the published RFC 4648 base16 vectors for '', 'f', 'fo', 'foo', 'foob', 'fooba', 'foobar' encode to their canonical uppercase-hex outputs"""
import base64

for _data, _expected in [
    (b"", b""),
    (b"f", b"66"),
    (b"fo", b"666F"),
    (b"foo", b"666F6F"),
    (b"foob", b"666F6F62"),
    (b"fooba", b"666F6F6261"),
    (b"foobar", b"666F6F626172"),
]:
    assert base64.b16encode(_data) == _expected, (_data, base64.b16encode(_data))
print("b16_rfc4648_vectors OK")
"###);
    assert_output(&out, r###"b16_rfc4648_vectors OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/b16_uppercase_hex_and_casefold.py`.
#[test]
fn test_gen_behavior_std_libs_base64_b16_uppercase_hex_and_casefold() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b16_uppercase_hex_and_casefold"
# subject = "base64.b16encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b16encode: b16encode emits uppercase hex (b'ABCDEF'); b16decode with casefold=True accepts lowercase input"""
import base64

assert base64.b16encode(b"\xab\xcd\xef") == b"ABCDEF", "b16 uppercase"
assert base64.b16decode(b"abcdef", casefold=True) == b"\xab\xcd\xef", "b16 casefold"
print("b16_uppercase_hex_and_casefold OK")
"###);
    assert_output(&out, r###"b16_uppercase_hex_and_casefold OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/b32_alphabet_and_round_trip.py`.
#[test]
fn test_gen_behavior_std_libs_base64_b32_alphabet_and_round_trip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b32_alphabet_and_round_trip"
# subject = "base64.b32encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b32encode: b32encode emits only the A-Z, 2-7, '=' alphabet and b32decode round-trips it back"""
import base64

_b32 = base64.b32encode(b"hello")
assert all(c in b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567=" for c in _b32), _b32
assert base64.b32decode(_b32) == b"hello", "b32 round-trip"
print("b32_alphabet_and_round_trip OK")
"###);
    assert_output(&out, r###"b32_alphabet_and_round_trip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/b32_rfc4648_vectors.py`.
#[test]
fn test_gen_behavior_std_libs_base64_b32_rfc4648_vectors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b32_rfc4648_vectors"
# subject = "base64.b32encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b32encode: the published RFC 4648 base32 vectors for '', 'f', 'fo', 'foo', 'foob', 'fooba', 'foobar' encode to their canonical outputs"""
import base64

for _data, _expected in [
    (b"", b""),
    (b"f", b"MY======"),
    (b"fo", b"MZXQ===="),
    (b"foo", b"MZXW6==="),
    (b"foob", b"MZXW6YQ="),
    (b"fooba", b"MZXW6YTB"),
    (b"foobar", b"MZXW6YTBOI======"),
]:
    assert base64.b32encode(_data) == _expected, (_data, base64.b32encode(_data))
print("b32_rfc4648_vectors OK")
"###);
    assert_output(&out, r###"b32_rfc4648_vectors OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/b32hex_rfc4648_vectors_and_round_trip.py`.
#[test]
fn test_gen_behavior_std_libs_base64_b32hex_rfc4648_vectors_and_round_trip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b32hex_rfc4648_vectors_and_round_trip"
# subject = "base64.b32hexencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b32hexencode: the RFC 4648 base32hex (extended-hex alphabet) vectors encode canonically and b32hexdecode reverses b32hexencode"""
import base64

for _data, _expected in [
    (b"", b""),
    (b"f", b"CO======"),
    (b"fo", b"CPNG===="),
    (b"foo", b"CPNMU==="),
    (b"foob", b"CPNMUOG="),
    (b"fooba", b"CPNMUOJ1"),
    (b"foobar", b"CPNMUOJ1E8======"),
]:
    assert base64.b32hexencode(_data) == _expected, (_data, base64.b32hexencode(_data))
assert base64.b32hexdecode(b"CPNMUOJ1") == b"fooba", "b32hex decode round-trip"
print("b32hex_rfc4648_vectors_and_round_trip OK")
"###);
    assert_output(&out, r###"b32hex_rfc4648_vectors_and_round_trip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/b64_all_bytes_round_trip.py`.
#[test]
fn test_gen_behavior_std_libs_base64_b64_all_bytes_round_trip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b64_all_bytes_round_trip"
# subject = "base64.b64encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b64encode: b64encode then b64decode round-trips every byte value 0..255 unchanged"""
import base64

_all_bytes = bytes(range(256))
_enc = base64.b64encode(_all_bytes)
assert base64.b64decode(_enc) == _all_bytes, "all bytes round-trip"
print("b64_all_bytes_round_trip OK")
"###);
    assert_output(&out, r###"b64_all_bytes_round_trip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/b64_altchars_round_trip.py`.
#[test]
fn test_gen_behavior_std_libs_base64_b64_altchars_round_trip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b64_altchars_round_trip"
# subject = "base64.b64encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b64encode: b64encode/b64decode honor a custom altchars=b'-_' two-char alphabet and round-trip through it"""
import base64

_enc_alt = base64.b64encode(b"\xfb\xef", altchars=b"-_")
assert base64.b64decode(_enc_alt, altchars=b"-_") == b"\xfb\xef", "altchars round-trip"
print("b64_altchars_round_trip OK")
"###);
    assert_output(&out, r###"b64_altchars_round_trip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/b64_padding_by_remainder.py`.
#[test]
fn test_gen_behavior_std_libs_base64_b64_padding_by_remainder() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b64_padding_by_remainder"
# subject = "base64.b64encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b64encode: b64 padding follows the 3-byte group remainder: 1 byte -> 'YQ==', 2 bytes -> 'YWI=', 3 bytes -> 'YWJj' (no padding)"""
import base64

# 1 byte -> 2 base64 chars + 2 padding
assert base64.b64encode(b"a") == b"YQ==", base64.b64encode(b"a")
# 2 bytes -> 3 base64 chars + 1 padding
assert base64.b64encode(b"ab") == b"YWI=", base64.b64encode(b"ab")
# 3 bytes -> 4 base64 chars, no padding
assert base64.b64encode(b"abc") == b"YWJj", base64.b64encode(b"abc")
print("b64_padding_by_remainder OK")
"###);
    assert_output(&out, r###"b64_padding_by_remainder OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/b64_rfc4648_vectors.py`.
#[test]
fn test_gen_behavior_std_libs_base64_b64_rfc4648_vectors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b64_rfc4648_vectors"
# subject = "base64.b64encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b64encode: the published RFC 4648 base64 vectors for '', 'f', 'fo', 'foo', 'foob', 'fooba', 'foobar' encode to their canonical outputs"""
import base64

for _data, _expected in [
    (b"", b""),
    (b"f", b"Zg=="),
    (b"fo", b"Zm8="),
    (b"foo", b"Zm9v"),
    (b"foob", b"Zm9vYg=="),
    (b"fooba", b"Zm9vYmE="),
    (b"foobar", b"Zm9vYmFy"),
]:
    assert base64.b64encode(_data) == _expected, (_data, base64.b64encode(_data))
print("b64_rfc4648_vectors OK")
"###);
    assert_output(&out, r###"b64_rfc4648_vectors OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/b64decode_accepts_str_and_bytes.py`.
#[test]
fn test_gen_behavior_std_libs_base64_b64decode_accepts_str_and_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b64decode_accepts_str_and_bytes"
# subject = "base64.b64decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b64decode: b64decode accepts both a bytes and a str payload, decoding 'aGVsbG8=' to b'hello' either way"""
import base64

assert base64.b64decode(b"aGVsbG8=") == b"hello", "bytes payload"
assert base64.b64decode("aGVsbG8=") == b"hello", "str payload"
print("b64decode_accepts_str_and_bytes OK")
"###);
    assert_output(&out, r###"b64decode_accepts_str_and_bytes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/b64decode_ignores_whitespace_by_default.py`.
#[test]
fn test_gen_behavior_std_libs_base64_b64decode_ignores_whitespace_by_default() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b64decode_ignores_whitespace_by_default"
# subject = "base64.b64decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b64decode: with validate=False (default) b64decode ignores embedded whitespace, decoding b'aGVsbG8=\\n' to b'hello'"""
import base64

assert base64.b64decode(b"aGVsbG8=\n") == b"hello", "decode ignores whitespace"
print("b64decode_ignores_whitespace_by_default OK")
"###);
    assert_output(&out, r###"b64decode_ignores_whitespace_by_default OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/b64encode_is_deterministic.py`.
#[test]
fn test_gen_behavior_std_libs_base64_b64encode_is_deterministic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b64encode_is_deterministic"
# subject = "base64.b64encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b64encode: b64encode is a pure function: the same input always produces the same output"""
import base64

_d = b"test data 123"
assert base64.b64encode(_d) == base64.b64encode(_d), "deterministic"
print("b64encode_is_deterministic OK")
"###);
    assert_output(&out, r###"b64encode_is_deterministic OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/b85_all_bytes_round_trip.py`.
#[test]
fn test_gen_behavior_std_libs_base64_b85_all_bytes_round_trip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b85_all_bytes_round_trip"
# subject = "base64.b85encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b85encode: b85encode then b85decode round-trips every byte value 0..255 unchanged, and empty input round-trips to empty bytes"""
import base64

_payload = bytes(range(256))
_enc = base64.b85encode(_payload)
assert isinstance(_enc, bytes), type(_enc)
assert base64.b85decode(_enc) == _payload, "b85 round-trip"
# Empty input round-trips to empty bytes.
assert base64.b85encode(b"") == b"", "b85 empty encode"
assert base64.b85decode(b"") == b"", "b85 empty decode"
print("b85_all_bytes_round_trip OK")
"###);
    assert_output(&out, r###"b85_all_bytes_round_trip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/base_xy_test_case__test_b32decode_error.py`.
#[test]
fn test_gen_behavior_std_libs_base64_base_xy_test_case__test_b32decode_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "base_xy_test_case__test_b32decode_error"
# subject = "cpython.test_base64.BaseXYTestCase.test_b32decode_error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_base64.py::BaseXYTestCase::test_b32decode_error
"""Auto-ported test: BaseXYTestCase::test_b32decode_error (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---
def check_decode_type_errors(f):

    try:
        f([])
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

def check_encode_type_errors(f):

    try:
        f('')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        f([])
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

def check_multidimensional(f, data):
    padding = b'\x00' if len(data) % 2 else b''
    bytes_data = data + padding
    shape = (len(bytes_data) // 2, 2)
    multidimensional = memoryview(bytes_data).cast('B', shape)

    assert f(multidimensional) == f(bytes_data)

def check_nonbyte_element_format(f, data):
    padding = b'\x00' * ((4 - len(data)) % 4)
    bytes_data = data + padding
    int_data = memoryview(bytes_data).cast('I')

    assert f(int_data) == f(bytes_data)

def check_other_types(f, bytes_data, expected):
    eq = self_assertEqual
    b = bytearray(bytes_data)
    eq(f(b), expected)
    eq(b, bytes_data)
    eq(f(memoryview(bytes_data)), expected)
    eq(f(array('B', bytes_data)), expected)
    check_nonbyte_element_format(base64.b64encode, bytes_data)
    check_multidimensional(base64.b64encode, bytes_data)
tests = [b'abc', b'ABCDEF==', b'==ABCDEF']
prefixes = [b'M', b'ME', b'MFRA', b'MFRGG', b'MFRGGZA', b'MFRGGZDF']
for i in range(0, 17):
    if i:
        tests.append(b'=' * i)
    for prefix in prefixes:
        if len(prefix) + i != 8:
            tests.append(prefix + b'=' * i)
for data in tests:
    try:
        base64.b32decode(data)
        raise AssertionError('expected binascii.Error')
    except binascii.Error:
        pass
    try:
        base64.b32decode(data.decode('ascii'))
        raise AssertionError('expected binascii.Error')
    except binascii.Error:
        pass
print("BaseXYTestCase::test_b32decode_error: ok")
"###);
    assert_output(&out, r###"BaseXYTestCase::test_b32decode_error: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/base_xy_test_case__test_b32hexdecode.py`.
#[test]
fn test_gen_behavior_std_libs_base64_base_xy_test_case__test_b32hexdecode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "base_xy_test_case__test_b32hexdecode"
# subject = "cpython.test_base64.BaseXYTestCase.test_b32hexdecode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_base64.py::BaseXYTestCase::test_b32hexdecode
"""Auto-ported test: BaseXYTestCase::test_b32hexdecode (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---
def check_decode_type_errors(f):

    try:
        f([])
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

def check_encode_type_errors(f):

    try:
        f('')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        f([])
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

def check_multidimensional(f, data):
    padding = b'\x00' if len(data) % 2 else b''
    bytes_data = data + padding
    shape = (len(bytes_data) // 2, 2)
    multidimensional = memoryview(bytes_data).cast('B', shape)

    assert f(multidimensional) == f(bytes_data)

def check_nonbyte_element_format(f, data):
    padding = b'\x00' * ((4 - len(data)) % 4)
    bytes_data = data + padding
    int_data = memoryview(bytes_data).cast('I')

    assert f(int_data) == f(bytes_data)

def check_other_types(f, bytes_data, expected):
    eq = self_assertEqual
    b = bytearray(bytes_data)
    eq(f(b), expected)
    eq(b, bytes_data)
    eq(f(memoryview(bytes_data)), expected)
    eq(f(array('B', bytes_data)), expected)
    check_nonbyte_element_format(base64.b64encode, bytes_data)
    check_multidimensional(base64.b64encode, bytes_data)
test_cases = [(b'', b'', False), (b'00======', b'\x00', False), (b'C4======', b'a', False), (b'C5H0====', b'ab', False), (b'C5H66===', b'abc', False), (b'C5H66P0=', b'abcd', False), (b'C5H66P35', b'abcde', False), (b'', b'', True), (b'00======', b'\x00', True), (b'C4======', b'a', True), (b'C5H0====', b'ab', True), (b'C5H66===', b'abc', True), (b'C5H66P0=', b'abcd', True), (b'C5H66P35', b'abcde', True), (b'c4======', b'a', True), (b'c5h0====', b'ab', True), (b'c5h66===', b'abc', True), (b'c5h66p0=', b'abcd', True), (b'c5h66p35', b'abcde', True)]
for to_decode, expected, casefold in test_cases:

    assert base64.b32hexdecode(to_decode, casefold) == expected

    assert base64.b32hexdecode(to_decode.decode('ascii'), casefold) == expected
print("BaseXYTestCase::test_b32hexdecode: ok")
"###);
    assert_output(&out, r###"BaseXYTestCase::test_b32hexdecode: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/base_xy_test_case__test_b32hexdecode_error.py`.
#[test]
fn test_gen_behavior_std_libs_base64_base_xy_test_case__test_b32hexdecode_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "base_xy_test_case__test_b32hexdecode_error"
# subject = "cpython.test_base64.BaseXYTestCase.test_b32hexdecode_error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_base64.py::BaseXYTestCase::test_b32hexdecode_error
"""Auto-ported test: BaseXYTestCase::test_b32hexdecode_error (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---
def check_decode_type_errors(f):

    try:
        f([])
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

def check_encode_type_errors(f):

    try:
        f('')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        f([])
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

def check_multidimensional(f, data):
    padding = b'\x00' if len(data) % 2 else b''
    bytes_data = data + padding
    shape = (len(bytes_data) // 2, 2)
    multidimensional = memoryview(bytes_data).cast('B', shape)

    assert f(multidimensional) == f(bytes_data)

def check_nonbyte_element_format(f, data):
    padding = b'\x00' * ((4 - len(data)) % 4)
    bytes_data = data + padding
    int_data = memoryview(bytes_data).cast('I')

    assert f(int_data) == f(bytes_data)

def check_other_types(f, bytes_data, expected):
    eq = self_assertEqual
    b = bytearray(bytes_data)
    eq(f(b), expected)
    eq(b, bytes_data)
    eq(f(memoryview(bytes_data)), expected)
    eq(f(array('B', bytes_data)), expected)
    check_nonbyte_element_format(base64.b64encode, bytes_data)
    check_multidimensional(base64.b64encode, bytes_data)
tests = [b'abc', b'ABCDEF==', b'==ABCDEF', b'c4======']
prefixes = [b'M', b'ME', b'MFRA', b'MFRGG', b'MFRGGZA', b'MFRGGZDF']
for i in range(0, 17):
    if i:
        tests.append(b'=' * i)
    for prefix in prefixes:
        if len(prefix) + i != 8:
            tests.append(prefix + b'=' * i)
for data in tests:
    try:
        base64.b32hexdecode(data)
        raise AssertionError('expected binascii.Error')
    except binascii.Error:
        pass
    try:
        base64.b32hexdecode(data.decode('ascii'))
        raise AssertionError('expected binascii.Error')
    except binascii.Error:
        pass
print("BaseXYTestCase::test_b32hexdecode_error: ok")
"###);
    assert_output(&out, r###"BaseXYTestCase::test_b32hexdecode_error: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/base_xy_test_case__test_b32hexencode.py`.
#[test]
fn test_gen_behavior_std_libs_base64_base_xy_test_case__test_b32hexencode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "base_xy_test_case__test_b32hexencode"
# subject = "cpython.test_base64.BaseXYTestCase.test_b32hexencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_base64.py::BaseXYTestCase::test_b32hexencode
"""Auto-ported test: BaseXYTestCase::test_b32hexencode (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---
def check_decode_type_errors(f):

    try:
        f([])
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

def check_encode_type_errors(f):

    try:
        f('')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        f([])
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

def check_multidimensional(f, data):
    padding = b'\x00' if len(data) % 2 else b''
    bytes_data = data + padding
    shape = (len(bytes_data) // 2, 2)
    multidimensional = memoryview(bytes_data).cast('B', shape)

    assert f(multidimensional) == f(bytes_data)

def check_nonbyte_element_format(f, data):
    padding = b'\x00' * ((4 - len(data)) % 4)
    bytes_data = data + padding
    int_data = memoryview(bytes_data).cast('I')

    assert f(int_data) == f(bytes_data)

def check_other_types(f, bytes_data, expected):
    eq = self_assertEqual
    b = bytearray(bytes_data)
    eq(f(b), expected)
    eq(b, bytes_data)
    eq(f(memoryview(bytes_data)), expected)
    eq(f(array('B', bytes_data)), expected)
    check_nonbyte_element_format(base64.b64encode, bytes_data)
    check_multidimensional(base64.b64encode, bytes_data)
test_cases = [(b'', b''), (b'\x00', b'00======'), (b'a', b'C4======'), (b'ab', b'C5H0===='), (b'abc', b'C5H66==='), (b'abcd', b'C5H66P0='), (b'abcde', b'C5H66P35')]
for to_encode, expected in test_cases:

    assert base64.b32hexencode(to_encode) == expected
print("BaseXYTestCase::test_b32hexencode: ok")
"###);
    assert_output(&out, r###"BaseXYTestCase::test_b32hexencode: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/base_xy_test_case__test_b64decode_invalid_chars.py`.
#[test]
fn test_gen_behavior_std_libs_base64_base_xy_test_case__test_b64decode_invalid_chars() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "base_xy_test_case__test_b64decode_invalid_chars"
# subject = "cpython.test_base64.BaseXYTestCase.test_b64decode_invalid_chars"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_base64.py::BaseXYTestCase::test_b64decode_invalid_chars
"""Auto-ported test: BaseXYTestCase::test_b64decode_invalid_chars (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---
def check_decode_type_errors(f):

    try:
        f([])
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

def check_encode_type_errors(f):

    try:
        f('')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        f([])
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

def check_multidimensional(f, data):
    padding = b'\x00' if len(data) % 2 else b''
    bytes_data = data + padding
    shape = (len(bytes_data) // 2, 2)
    multidimensional = memoryview(bytes_data).cast('B', shape)

    assert f(multidimensional) == f(bytes_data)

def check_nonbyte_element_format(f, data):
    padding = b'\x00' * ((4 - len(data)) % 4)
    bytes_data = data + padding
    int_data = memoryview(bytes_data).cast('I')

    assert f(int_data) == f(bytes_data)

def check_other_types(f, bytes_data, expected):
    eq = self_assertEqual
    b = bytearray(bytes_data)
    eq(f(b), expected)
    eq(b, bytes_data)
    eq(f(memoryview(bytes_data)), expected)
    eq(f(array('B', bytes_data)), expected)
    check_nonbyte_element_format(base64.b64encode, bytes_data)
    check_multidimensional(base64.b64encode, bytes_data)
tests = ((b'%3d==', b'\xdd'), (b'$3d==', b'\xdd'), (b'[==', b''), (b'YW]3=', b'am'), (b'3{d==', b'\xdd'), (b'3d}==', b'\xdd'), (b'@@', b''), (b'!', b''), (b'YWJj\n', b'abc'), (b'YWJj\nYWI=', b'abcab'))
funcs = (base64.b64decode, base64.standard_b64decode, base64.urlsafe_b64decode)
for bstr, res in tests:
    for func in funcs:

        assert func(bstr) == res

        assert func(bstr.decode('ascii')) == res
    try:
        base64.b64decode(bstr, validate=True)
        raise AssertionError('expected binascii.Error')
    except binascii.Error:
        pass
    try:
        base64.b64decode(bstr.decode('ascii'), validate=True)
        raise AssertionError('expected binascii.Error')
    except binascii.Error:
        pass
res = b'\xfb\xef\xbe\xff\xff\xff'

assert base64.b64decode(b'++[[//]]', b'[]') == res

assert base64.urlsafe_b64decode(b'++--//__') == res
print("BaseXYTestCase::test_b64decode_invalid_chars: ok")
"###);
    assert_output(&out, r###"BaseXYTestCase::test_b64decode_invalid_chars: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/base_xy_test_case__test_b64decode_padding_error.py`.
#[test]
fn test_gen_behavior_std_libs_base64_base_xy_test_case__test_b64decode_padding_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "base_xy_test_case__test_b64decode_padding_error"
# subject = "cpython.test_base64.BaseXYTestCase.test_b64decode_padding_error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_base64.py::BaseXYTestCase::test_b64decode_padding_error
"""Auto-ported test: BaseXYTestCase::test_b64decode_padding_error (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---

try:
    base64.b64decode(b'abc')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    base64.b64decode('abc')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass
print("BaseXYTestCase::test_b64decode_padding_error: ok")
"###);
    assert_output(&out, r###"BaseXYTestCase::test_b64decode_padding_error: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/base_xy_test_case__test_b85decode_errors.py`.
#[test]
fn test_gen_behavior_std_libs_base64_base_xy_test_case__test_b85decode_errors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "base_xy_test_case__test_b85decode_errors"
# subject = "cpython.test_base64.BaseXYTestCase.test_b85decode_errors"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_base64.py::BaseXYTestCase::test_b85decode_errors
"""Auto-ported test: BaseXYTestCase::test_b85decode_errors (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---
illegal = list(range(33)) + list(b'"\',./:[\\]') + list(range(128, 256))
for c in illegal:
    try:
        base64.b85decode(b'0000' + bytes([c]))
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

try:
    base64.b85decode(b'|')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.b85decode(b'|N')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.b85decode(b'|Ns')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.b85decode(b'|NsC')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.b85decode(b'|NsC1')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("BaseXYTestCase::test_b85decode_errors: ok")
"###);
    assert_output(&out, r###"BaseXYTestCase::test_b85decode_errors: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/base_xy_test_case__test_decode_nonascii_str.py`.
#[test]
fn test_gen_behavior_std_libs_base64_base_xy_test_case__test_decode_nonascii_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "base_xy_test_case__test_decode_nonascii_str"
# subject = "cpython.test_base64.BaseXYTestCase.test_decode_nonascii_str"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_base64.py::BaseXYTestCase::test_decode_nonascii_str
"""Auto-ported test: BaseXYTestCase::test_decode_nonascii_str (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---
decode_funcs = (base64.b64decode, base64.standard_b64decode, base64.urlsafe_b64decode, base64.b32decode, base64.b16decode, base64.b85decode, base64.a85decode)
for f in decode_funcs:

    try:
        f('with non-ascii Ë')
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
print("BaseXYTestCase::test_decode_nonascii_str: ok")
"###);
    assert_output(&out, r###"BaseXYTestCase::test_decode_nonascii_str: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/base_xy_test_case__test_error_heritage.py`.
#[test]
fn test_gen_behavior_std_libs_base64_base_xy_test_case__test_error_heritage() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "base_xy_test_case__test_error_heritage"
# subject = "cpython.test_base64.BaseXYTestCase.test_ErrorHeritage"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_base64.py::BaseXYTestCase::test_ErrorHeritage
"""Auto-ported test: BaseXYTestCase::test_ErrorHeritage (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---

assert issubclass(binascii.Error, ValueError)
print("BaseXYTestCase::test_ErrorHeritage: ok")
"###);
    assert_output(&out, r###"BaseXYTestCase::test_ErrorHeritage: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/base_xy_test_case__test_rfc4648_test_cases.py`.
#[test]
fn test_gen_behavior_std_libs_base64_base_xy_test_case__test_rfc4648_test_cases() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "base_xy_test_case__test_rfc4648_test_cases"
# subject = "cpython.test_base64.BaseXYTestCase.test_RFC4648_test_cases"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_base64.py::BaseXYTestCase::test_RFC4648_test_cases
"""Auto-ported test: BaseXYTestCase::test_RFC4648_test_cases (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---
b64encode = base64.b64encode
b32hexencode = base64.b32hexencode
b32encode = base64.b32encode
b16encode = base64.b16encode

assert b64encode(b'') == b''

assert b64encode(b'f') == b'Zg=='

assert b64encode(b'fo') == b'Zm8='

assert b64encode(b'foo') == b'Zm9v'

assert b64encode(b'foob') == b'Zm9vYg=='

assert b64encode(b'fooba') == b'Zm9vYmE='

assert b64encode(b'foobar') == b'Zm9vYmFy'

assert b32encode(b'') == b''

assert b32encode(b'f') == b'MY======'

assert b32encode(b'fo') == b'MZXQ===='

assert b32encode(b'foo') == b'MZXW6==='

assert b32encode(b'foob') == b'MZXW6YQ='

assert b32encode(b'fooba') == b'MZXW6YTB'

assert b32encode(b'foobar') == b'MZXW6YTBOI======'

assert b32hexencode(b'') == b''

assert b32hexencode(b'f') == b'CO======'

assert b32hexencode(b'fo') == b'CPNG===='

assert b32hexencode(b'foo') == b'CPNMU==='

assert b32hexencode(b'foob') == b'CPNMUOG='

assert b32hexencode(b'fooba') == b'CPNMUOJ1'

assert b32hexencode(b'foobar') == b'CPNMUOJ1E8======'

assert b16encode(b'') == b''

assert b16encode(b'f') == b'66'

assert b16encode(b'fo') == b'666F'

assert b16encode(b'foo') == b'666F6F'

assert b16encode(b'foob') == b'666F6F62'

assert b16encode(b'fooba') == b'666F6F6261'

assert b16encode(b'foobar') == b'666F6F626172'
print("BaseXYTestCase::test_RFC4648_test_cases: ok")
"###);
    assert_output(&out, r###"BaseXYTestCase::test_RFC4648_test_cases: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/encodebytes_wraps_at_76_columns.py`.
#[test]
fn test_gen_behavior_std_libs_base64_encodebytes_wraps_at_76_columns() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "encodebytes_wraps_at_76_columns"
# subject = "base64.encodebytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.encodebytes: encodebytes inserts newlines so every line is at most 76 chars, and decodebytes round-trips the wrapped output back to the original"""
import base64

_long = b"x" * 100
_eb = base64.encodebytes(_long)
assert b"\n" in _eb, "encodebytes wraps with newlines"
for _line in _eb.split(b"\n"):
    assert len(_line) <= 76, len(_line)
assert base64.decodebytes(_eb) == _long, "decodebytes round-trip"
print("encodebytes_wraps_at_76_columns OK")
"###);
    assert_output(&out, r###"encodebytes_wraps_at_76_columns OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/legacy_stream_encode_decode_round_trip.py`.
#[test]
fn test_gen_behavior_std_libs_base64_legacy_stream_encode_decode_round_trip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "legacy_stream_encode_decode_round_trip"
# subject = "base64.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.encode: the legacy base64.encode/base64.decode file-object API line-wraps to base64 and back through in-memory BytesIO streams"""
import base64
from io import BytesIO

# encode() reads raw bytes and writes line-wrapped base64 bytes.
_in = BytesIO(b"www.python.org")
_out = BytesIO()
base64.encode(_in, _out)
assert _out.getvalue() == b"d3d3LnB5dGhvbi5vcmc=\n", _out.getvalue()

# decode() reads base64 bytes and writes the decoded bytes back.
_in2 = BytesIO(_out.getvalue())
_out2 = BytesIO()
base64.decode(_in2, _out2)
assert _out2.getvalue() == b"www.python.org", _out2.getvalue()
print("legacy_stream_encode_decode_round_trip OK")
"###);
    assert_output(&out, r###"legacy_stream_encode_decode_round_trip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/urlsafe_alphabet_has_no_plus_or_slash.py`.
#[test]
fn test_gen_behavior_std_libs_base64_urlsafe_alphabet_has_no_plus_or_slash() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "urlsafe_alphabet_has_no_plus_or_slash"
# subject = "base64.urlsafe_b64encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.urlsafe_b64encode: urlsafe_b64encode never emits '+' or '/' even when round-tripping every byte value 0..255"""
import base64

_all_bytes = bytes(range(256))
_url = base64.urlsafe_b64encode(_all_bytes)
assert b"+" not in _url, "urlsafe no +"
assert b"/" not in _url, "urlsafe no /"
assert base64.urlsafe_b64decode(_url) == _all_bytes, "urlsafe round-trip"
print("urlsafe_alphabet_has_no_plus_or_slash OK")
"###);
    assert_output(&out, r###"urlsafe_alphabet_has_no_plus_or_slash OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/base64/urlsafe_differs_from_standard.py`.
#[test]
fn test_gen_behavior_std_libs_base64_urlsafe_differs_from_standard() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "urlsafe_differs_from_standard"
# subject = "base64.urlsafe_b64encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.urlsafe_b64encode: urlsafe_b64encode replaces '+'/'/' with '-'/'_' so its output differs from standard b64encode for bytes that hit those positions, and still round-trips"""
import base64

# These bytes encode to a group containing '+' and '/' under the standard
# alphabet, so the urlsafe alphabet must differ.
_payload = b"\xfb\xff"
_std = base64.b64encode(_payload)
_url = base64.urlsafe_b64encode(_payload)
assert _std != _url, (_std, _url)
assert base64.urlsafe_b64decode(_url) == _payload, "urlsafe round-trip"
print("urlsafe_differs_from_standard OK")
"###);
    assert_output(&out, r###"urlsafe_differs_from_standard OK
"###);
}
