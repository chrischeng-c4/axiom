# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "checksum_big_buffer_test_case__test_big_buffer"
# subject = "cpython.test_binascii.ChecksumBigBufferTestCase.test_big_buffer"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_binascii.py::ChecksumBigBufferTestCase::test_big_buffer
"""Auto-ported test: ChecksumBigBufferTestCase::test_big_buffer (CPython 3.12 oracle)."""


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
data = b'nyan' * (_1G + 1)

assert binascii.crc32(data) == 1044521549
print("ChecksumBigBufferTestCase::test_big_buffer: ok")
