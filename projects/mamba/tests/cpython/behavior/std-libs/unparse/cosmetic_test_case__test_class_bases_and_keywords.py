# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unparse"
# dimension = "behavior"
# case = "cosmetic_test_case__test_class_bases_and_keywords"
# subject = "cpython.test_unparse.CosmeticTestCase.test_class_bases_and_keywords"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_unparse
_suite = unittest.defaultTestLoader.loadTestsFromName("CosmeticTestCase.test_class_bases_and_keywords", test_unparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CosmeticTestCase.test_class_bases_and_keywords did not pass"
print("CosmeticTestCase::test_class_bases_and_keywords: ok")
