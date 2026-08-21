# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "memoryview_bin_ascii_test__test_uu"
# subject = "cpython.test_binascii.MemoryviewBinASCIITest.test_uu"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::MemoryviewBinASCIITest::test_uu
"""Auto-ported test: MemoryviewBinASCIITest::test_uu (CPython 3.12 oracle)."""


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
MAX_UU = 45
for backtick in (True, False):
    lines = []
    for i in range(0, len(self_data), MAX_UU):
        b = type2test(rawdata[i:i + MAX_UU])
        a = binascii.b2a_uu(b, backtick=backtick)
        lines.append(a)
    res = bytes()
    for line in lines:
        a = type2test(line)
        b = binascii.a2b_uu(a)
        res += b

    assert res == rawdata

assert binascii.a2b_uu(b'\x7f') == b'\x00' * 31

assert binascii.a2b_uu(b'\x80') == b'\x00' * 32

assert binascii.a2b_uu(b'\xff') == b'\x00' * 31

try:
    binascii.a2b_uu(b'\xff\x00')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.a2b_uu(b'!!!!')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    binascii.b2a_uu(46 * b'!')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

assert binascii.b2a_uu(b'x') == b'!>   \n'

assert binascii.b2a_uu(b'') == b' \n'

assert binascii.b2a_uu(b'', backtick=True) == b'`\n'

assert binascii.a2b_uu(b' \n') == b''

assert binascii.a2b_uu(b'`\n') == b''

assert binascii.b2a_uu(b'\x00Cat') == b'$ $-A=   \n'

assert binascii.b2a_uu(b'\x00Cat', backtick=True) == b'$`$-A=```\n'

assert binascii.a2b_uu(b'$`$-A=```\n') == binascii.a2b_uu(b'$ $-A=   \n')
try:
    binascii.b2a_uu(b'', True)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("MemoryviewBinASCIITest::test_uu: ok")
