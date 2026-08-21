# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bin_ascii_test__test_hex"
# subject = "cpython.test_binascii.BinASCIITest.test_hex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BinASCIITest::test_hex
"""Auto-ported test: BinASCIITest::test_hex (CPython 3.12 oracle)."""


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
self_data = type2test(rawdata)
s = b'{s\x05\x00\x00\x00worldi\x02\x00\x00\x00s\x05\x00\x00\x00helloi\x01\x00\x00\x000'
t = binascii.b2a_hex(type2test(s))
u = binascii.a2b_hex(type2test(t))

assert s == u

try:
    binascii.a2b_hex(t[:-1])
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(t[:-1] + b'q')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(bytes([255, 255]))
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(b'0G')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(b'0g')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(b'G0')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_hex(b'g0')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

assert binascii.hexlify(type2test(s)) == t

assert binascii.unhexlify(type2test(t)) == u
print("BinASCIITest::test_hex: ok")
