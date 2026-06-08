# /// script
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
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
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
