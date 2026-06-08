# /// script
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
