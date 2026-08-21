# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "bigmem"
# dimension = "behavior"
# case = "str_test__test_encode_utf32"
# subject = "cpython.test_bigmem.StrTest.test_encode_utf32"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bigmem.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bigmem
_suite = unittest.defaultTestLoader.loadTestsFromName("StrTest.test_encode_utf32", test_bigmem)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StrTest.test_encode_utf32 did not pass"
print("StrTest::test_encode_utf32: ok")
