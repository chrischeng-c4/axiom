# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "array_bin_ascii_test__test_exceptions"
# subject = "cpython.test_binascii.ArrayBinASCIITest.test_exceptions"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::ArrayBinASCIITest::test_exceptions
"""Auto-ported test: ArrayBinASCIITest::test_exceptions (CPython 3.12 oracle)."""


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

assert issubclass(binascii.Error, Exception)

assert issubclass(binascii.Incomplete, Exception)
print("ArrayBinASCIITest::test_exceptions: ok")
