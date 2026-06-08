# /// script
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
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
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
