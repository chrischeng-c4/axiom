# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fixers"
# dimension = "behavior"
# case = "test_imports_fixer_order__test_after_local_imports_refactoring"
# subject = "cpython.test_fixers.Test_imports_fixer_order.test_after_local_imports_refactoring"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_fixers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_fixers
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_imports_fixer_order.test_after_local_imports_refactoring", test_fixers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_imports_fixer_order.test_after_local_imports_refactoring did not pass"
print("Test_imports_fixer_order::test_after_local_imports_refactoring: ok")
