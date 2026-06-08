# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "memoryview_bin_ascii_test__test_unicode_a2b"
# subject = "cpython.test_binascii.MemoryviewBinASCIITest.test_unicode_a2b"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_binascii.py::MemoryviewBinASCIITest::test_unicode_a2b
"""Auto-ported test: MemoryviewBinASCIITest::test_unicode_a2b (CPython 3.12 oracle)."""


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
MAX_ALL = 45
raw = rawdata[:MAX_ALL]
for fa, fb in zip(a2b_functions, b2a_functions):
    a2b = getattr(binascii, fa)
    b2a = getattr(binascii, fb)
    try:
        a = b2a(type2test(raw))
        binary_res = a2b(a)
        a = a.decode('ascii')
        res = a2b(a)
    except Exception as err:

        raise AssertionError('{}/{} conversion raises {!r}'.format(fb, fa, err))

    assert res == raw

    assert res == binary_res

    assert isinstance(res, bytes)

    try:
        a2b('\x80')
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
print("MemoryviewBinASCIITest::test_unicode_a2b: ok")
