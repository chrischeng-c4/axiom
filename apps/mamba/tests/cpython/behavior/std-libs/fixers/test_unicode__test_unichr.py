# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fixers"
# dimension = "behavior"
# case = "test_unicode__test_unichr"
# subject = "cpython.test_fixers.Test_unicode.test_unichr"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_fixers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_fixers
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_unicode.test_unichr", test_fixers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_unicode.test_unichr did not pass"
print("Test_unicode::test_unichr: ok")
