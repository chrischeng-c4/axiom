# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bytearray_bin_ascii_test__test_hex_separator"
# subject = "cpython.test_binascii.BytearrayBinASCIITest.test_hex_separator"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_binascii.py::BytearrayBinASCIITest::test_hex_separator
"""Auto-ported test: BytearrayBinASCIITest::test_hex_separator (CPython 3.12 oracle)."""


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
'Test that hexlify and b2a_hex are binary versions of bytes.hex.'
s = b'{s\x05\x00\x00\x00worldi\x02\x00\x00\x00s\x05\x00\x00\x00helloi\x01\x00\x00\x000'

assert binascii.hexlify(type2test(s)) == s.hex().encode('ascii')
expected8 = s.hex('.', 8).encode('ascii')

assert binascii.hexlify(type2test(s), '.', 8) == expected8
expected1 = s.hex(':').encode('ascii')

assert binascii.b2a_hex(type2test(s), ':') == expected1
print("BytearrayBinASCIITest::test_hex_separator: ok")
