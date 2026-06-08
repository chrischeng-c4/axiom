# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bytearray_bin_ascii_test__test_crc_hqx"
# subject = "cpython.test_binascii.BytearrayBinASCIITest.test_crc_hqx"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_binascii.py::BytearrayBinASCIITest::test_crc_hqx
"""Auto-ported test: BytearrayBinASCIITest::test_crc_hqx (CPython 3.12 oracle)."""


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
crc = binascii.crc_hqx(type2test(b'Test the CRC-32 of'), 0)
crc = binascii.crc_hqx(type2test(b' this string.'), crc)

assert crc == 14290

try:
    binascii.crc_hqx()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    binascii.crc_hqx(type2test(b''))
    raise AssertionError('expected TypeError')
except TypeError:
    pass
for crc in (0, 1, 4660, 74565, 305419896, -1):

    assert binascii.crc_hqx(type2test(b''), crc) == crc & 65535
print("BytearrayBinASCIITest::test_crc_hqx: ok")
