# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "httpservers"
# dimension = "behavior"
# case = "misc_test_case__test_all"
# subject = "cpython.test_httpservers.MiscTestCase.test_all"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_httpservers
_suite = unittest.defaultTestLoader.loadTestsFromName("MiscTestCase.test_all", test_httpservers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MiscTestCase.test_all did not pass"
print("MiscTestCase::test_all: ok")
