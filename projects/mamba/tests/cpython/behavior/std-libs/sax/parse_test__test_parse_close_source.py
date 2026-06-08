# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sax"
# dimension = "behavior"
# case = "parse_test__test_parse_close_source"
# subject = "cpython.test_sax.ParseTest.test_parse_close_source"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sax.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sax
_suite = unittest.defaultTestLoader.loadTestsFromName("ParseTest.test_parse_close_source", test_sax)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ParseTest.test_parse_close_source did not pass"
print("ParseTest::test_parse_close_source: ok")
