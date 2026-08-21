# /// script
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
