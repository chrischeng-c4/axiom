# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "float"
# dimension = "behavior"
# case = "round_test_case__test_format_specials_ucf4965f"
# subject = "cpython.test_float.RoundTestCase.test_format_specials"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_float
_suite = unittest.defaultTestLoader.loadTestsFromName("RoundTestCase.test_format_specials", test_float)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RoundTestCase.test_format_specials did not pass"
print("RoundTestCase::test_format_specials: ok")
