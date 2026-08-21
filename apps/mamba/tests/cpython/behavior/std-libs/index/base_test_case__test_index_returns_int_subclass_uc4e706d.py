# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "index"
# dimension = "behavior"
# case = "base_test_case__test_index_returns_int_subclass_uc4e706d"
# subject = "cpython.test_index.BaseTestCase.test_index_returns_int_subclass"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_index.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_index
_suite = unittest.defaultTestLoader.loadTestsFromName("BaseTestCase.test_index_returns_int_subclass", test_index)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BaseTestCase.test_index_returns_int_subclass did not pass"
print("BaseTestCase::test_index_returns_int_subclass: ok")
