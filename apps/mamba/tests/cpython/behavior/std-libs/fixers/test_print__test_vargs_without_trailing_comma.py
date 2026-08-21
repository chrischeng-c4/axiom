# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fixers"
# dimension = "behavior"
# case = "test_print__test_vargs_without_trailing_comma"
# subject = "cpython.test_fixers.Test_print.test_vargs_without_trailing_comma"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_fixers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_fixers
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_print.test_vargs_without_trailing_comma", test_fixers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_print.test_vargs_without_trailing_comma did not pass"
print("Test_print::test_vargs_without_trailing_comma: ok")
