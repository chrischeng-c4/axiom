# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "source_encoding"
# dimension = "behavior"
# case = "misc_source_encoding_test__test_20731_uca8c064"
# subject = "cpython.test_source_encoding.MiscSourceEncodingTest.test_20731"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_source_encoding.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_source_encoding
_suite = unittest.defaultTestLoader.loadTestsFromName("MiscSourceEncodingTest.test_20731", test_source_encoding)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MiscSourceEncodingTest.test_20731 did not pass"
print("MiscSourceEncodingTest::test_20731: ok")
