# /// script
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
