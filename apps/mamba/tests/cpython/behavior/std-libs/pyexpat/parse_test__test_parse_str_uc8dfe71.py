# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyexpat"
# dimension = "behavior"
# case = "parse_test__test_parse_str_uc8dfe71"
# subject = "cpython.test_pyexpat.ParseTest.test_parse_str"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pyexpat.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pyexpat
_suite = unittest.defaultTestLoader.loadTestsFromName("ParseTest.test_parse_str", test_pyexpat)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ParseTest.test_parse_str did not pass"
print("ParseTest::test_parse_str: ok")
