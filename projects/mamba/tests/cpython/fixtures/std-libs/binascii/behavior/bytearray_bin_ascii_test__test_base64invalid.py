# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bytearray_bin_ascii_test__test_base64invalid"
# subject = "cpython.test_binascii.BytearrayBinASCIITest.test_base64invalid"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_binascii.py::BytearrayBinASCIITest::test_base64invalid
"""Auto-ported test: BytearrayBinASCIITest::test_base64invalid (CPython 3.12 oracle)."""


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
for i in range(0, len(self_data), MAX_BASE64):
    b = type2test(rawdata[i:i + MAX_BASE64])
    a = binascii.b2a_base64(b)
    lines.append(a)
fillers = bytearray()
valid = b'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/'
for i in range(256):
    if i not in valid:
        fillers.append(i)

def addnoise(line):
    noise = fillers
    ratio = len(line) // len(noise)
    res = bytearray()
    while line and noise:
        if len(line) // len(noise) > ratio:
            c, line = (line[0], line[1:])
        else:
            c, noise = (noise[0], noise[1:])
        res.append(c)
    return res + noise + line
res = bytearray()
for line in map(addnoise, lines):
    a = type2test(line)
    b = binascii.a2b_base64(a)
    res += b

assert res == rawdata

assert binascii.a2b_base64(type2test(fillers)) == b''
print("BytearrayBinASCIITest::test_base64invalid: ok")
