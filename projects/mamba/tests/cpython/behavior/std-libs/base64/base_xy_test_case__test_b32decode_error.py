# /// script
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
