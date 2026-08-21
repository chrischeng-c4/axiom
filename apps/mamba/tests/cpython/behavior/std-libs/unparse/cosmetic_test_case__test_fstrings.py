# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unparse"
# dimension = "behavior"
# case = "cosmetic_test_case__test_fstrings"
# subject = "cpython.test_unparse.CosmeticTestCase.test_fstrings"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_unparse
_suite = unittest.defaultTestLoader.loadTestsFromName("CosmeticTestCase.test_fstrings", test_unparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CosmeticTestCase.test_fstrings did not pass"
print("CosmeticTestCase::test_fstrings: ok")
