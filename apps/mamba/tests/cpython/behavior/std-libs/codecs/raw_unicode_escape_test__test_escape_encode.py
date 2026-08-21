# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "raw_unicode_escape_test__test_escape_encode"
# subject = "cpython.test_codecs.RawUnicodeEscapeTest.test_escape_encode"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_codecs
_suite = unittest.defaultTestLoader.loadTestsFromName("RawUnicodeEscapeTest.test_escape_encode", test_codecs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RawUnicodeEscapeTest.test_escape_encode did not pass"
print("RawUnicodeEscapeTest::test_escape_encode: ok")
