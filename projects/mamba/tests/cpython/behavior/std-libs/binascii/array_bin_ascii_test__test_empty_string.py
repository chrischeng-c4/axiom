# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "array_bin_ascii_test__test_empty_string"
# subject = "cpython.test_binascii.ArrayBinASCIITest.test_empty_string"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_binascii.py::ArrayBinASCIITest::test_empty_string
"""Auto-ported test: ArrayBinASCIITest::test_empty_string (CPython 3.12 oracle)."""


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
print("ArrayBinASCIITest::test_empty_string: ok")
