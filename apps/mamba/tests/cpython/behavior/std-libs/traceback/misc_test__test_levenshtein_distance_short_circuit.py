# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "misc_test__test_levenshtein_distance_short_circuit"
# subject = "cpython.test_traceback.MiscTest.test_levenshtein_distance_short_circuit"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_traceback
_suite = unittest.defaultTestLoader.loadTestsFromName("MiscTest.test_levenshtein_distance_short_circuit", test_traceback)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MiscTest.test_levenshtein_distance_short_circuit did not pass"
print("MiscTest::test_levenshtein_distance_short_circuit: ok")
