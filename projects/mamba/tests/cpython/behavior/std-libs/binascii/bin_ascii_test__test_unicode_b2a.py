# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bin_ascii_test__test_unicode_b2a"
# subject = "cpython.test_binascii.BinASCIITest.test_unicode_b2a"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_binascii.py::BinASCIITest::test_unicode_b2a
"""Auto-ported test: BinASCIITest::test_unicode_b2a (CPython 3.12 oracle)."""


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
for func in set(all_functions) - set(a2b_functions):
    try:

        try:
            getattr(binascii, func)('test')
            raise AssertionError('expected TypeError')
        except TypeError:
            pass
    except Exception as err:

        raise AssertionError('{}("test") raises {!r}'.format(func, err))

try:
    binascii.crc_hqx('test', 0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("BinASCIITest::test_unicode_b2a: ok")
