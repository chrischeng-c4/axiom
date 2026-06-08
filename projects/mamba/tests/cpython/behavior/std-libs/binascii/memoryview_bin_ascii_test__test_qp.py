# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "memoryview_bin_ascii_test__test_qp"
# subject = "cpython.test_binascii.MemoryviewBinASCIITest.test_qp"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_binascii.py::MemoryviewBinASCIITest::test_qp
"""Auto-ported test: MemoryviewBinASCIITest::test_qp (CPython 3.12 oracle)."""


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
type2test = type2test
a2b_qp = binascii.a2b_qp
b2a_qp = binascii.b2a_qp
a2b_qp(data=b'', header=False)
try:
    a2b_qp(b'', **{1: 1})
except TypeError:
    pass
else:

    raise AssertionError("binascii.a2b_qp(**{1:1}) didn't raise TypeError")

assert a2b_qp(type2test(b'=')) == b''

assert a2b_qp(type2test(b'= ')) == b'= '

assert a2b_qp(type2test(b'==')) == b'='

assert a2b_qp(type2test(b'=\nAB')) == b'AB'

assert a2b_qp(type2test(b'=\r\nAB')) == b'AB'

assert a2b_qp(type2test(b'=\rAB')) == b''

assert a2b_qp(type2test(b'=\rAB\nCD')) == b'CD'

assert a2b_qp(type2test(b'=AB')) == b'\xab'

assert a2b_qp(type2test(b'=ab')) == b'\xab'

assert a2b_qp(type2test(b'=AX')) == b'=AX'

assert a2b_qp(type2test(b'=XA')) == b'=XA'

assert a2b_qp(type2test(b'=AB')[:-1]) == b'=A'

assert a2b_qp(type2test(b'_')) == b'_'

assert a2b_qp(type2test(b'_'), header=True) == b' '

try:
    b2a_qp(foo='bar')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert a2b_qp(type2test(b'=00\r\n=00')) == b'\x00\r\n\x00'

assert b2a_qp(type2test(b'\xff\r\n\xff\n\xff')) == b'=FF\r\n=FF\r\n=FF'

assert b2a_qp(type2test(b'0' * 75 + b'\xff\r\n\xff\r\n\xff')) == b'0' * 75 + b'=\r\n=FF\r\n=FF\r\n=FF'

assert b2a_qp(type2test(b'\x7f')) == b'=7F'

assert b2a_qp(type2test(b'=')) == b'=3D'

assert b2a_qp(type2test(b'_')) == b'_'

assert b2a_qp(type2test(b'_'), header=True) == b'=5F'

assert b2a_qp(type2test(b'x y'), header=True) == b'x_y'

assert b2a_qp(type2test(b'x '), header=True) == b'x=20'

assert b2a_qp(type2test(b'x y'), header=True, quotetabs=True) == b'x=20y'

assert b2a_qp(type2test(b'x\ty'), header=True) == b'x\ty'

assert b2a_qp(type2test(b' ')) == b'=20'

assert b2a_qp(type2test(b'\t')) == b'=09'

assert b2a_qp(type2test(b' x')) == b' x'

assert b2a_qp(type2test(b'\tx')) == b'\tx'

assert b2a_qp(type2test(b' x')[:-1]) == b'=20'

assert b2a_qp(type2test(b'\tx')[:-1]) == b'=09'

assert b2a_qp(type2test(b'\x00')) == b'=00'

assert b2a_qp(type2test(b'\x00\n')) == b'=00\n'

assert b2a_qp(type2test(b'\x00\n'), quotetabs=True) == b'=00\n'

assert b2a_qp(type2test(b'x y\tz')) == b'x y\tz'

assert b2a_qp(type2test(b'x y\tz'), quotetabs=True) == b'x=20y=09z'

assert b2a_qp(type2test(b'x y\tz'), istext=False) == b'x y\tz'

assert b2a_qp(type2test(b'x \ny\t\n')) == b'x=20\ny=09\n'

assert b2a_qp(type2test(b'x \ny\t\n'), quotetabs=True) == b'x=20\ny=09\n'

assert b2a_qp(type2test(b'x \ny\t\n'), istext=False) == b'x =0Ay\t=0A'

assert b2a_qp(type2test(b'x \ry\t\r')) == b'x \ry\t\r'

assert b2a_qp(type2test(b'x \ry\t\r'), quotetabs=True) == b'x=20\ry=09\r'

assert b2a_qp(type2test(b'x \ry\t\r'), istext=False) == b'x =0Dy\t=0D'

assert b2a_qp(type2test(b'x \r\ny\t\r\n')) == b'x=20\r\ny=09\r\n'

assert b2a_qp(type2test(b'x \r\ny\t\r\n'), quotetabs=True) == b'x=20\r\ny=09\r\n'

assert b2a_qp(type2test(b'x \r\ny\t\r\n'), istext=False) == b'x =0D=0Ay\t=0D=0A'

assert b2a_qp(type2test(b'x \r\n')[:-1]) == b'x \r'

assert b2a_qp(type2test(b'x\t\r\n')[:-1]) == b'x\t\r'

assert b2a_qp(type2test(b'x \r\n')[:-1], quotetabs=True) == b'x=20\r'

assert b2a_qp(type2test(b'x\t\r\n')[:-1], quotetabs=True) == b'x=09\r'

assert b2a_qp(type2test(b'x \r\n')[:-1], istext=False) == b'x =0D'

assert b2a_qp(type2test(b'x\t\r\n')[:-1], istext=False) == b'x\t=0D'

assert b2a_qp(type2test(b'.')) == b'=2E'

assert b2a_qp(type2test(b'.\n')) == b'=2E\n'

assert b2a_qp(type2test(b'.\r')) == b'=2E\r'

assert b2a_qp(type2test(b'.\x00')) == b'=2E=00'

assert b2a_qp(type2test(b'a.\n')) == b'a.\n'

assert b2a_qp(type2test(b'.a')[:-1]) == b'=2E'
print("MemoryviewBinASCIITest::test_qp: ok")
