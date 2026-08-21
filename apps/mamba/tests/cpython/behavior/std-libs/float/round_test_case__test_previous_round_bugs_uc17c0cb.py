# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "float"
# dimension = "behavior"
# case = "round_test_case__test_previous_round_bugs_uc17c0cb"
# subject = "cpython.test_float.RoundTestCase.test_previous_round_bugs"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_float
_suite = unittest.defaultTestLoader.loadTestsFromName("RoundTestCase.test_previous_round_bugs", test_float)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RoundTestCase.test_previous_round_bugs did not pass"
print("RoundTestCase::test_previous_round_bugs: ok")
