# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "refcounts"
# dimension = "behavior"
# case = "refcount_test_case__test_1_uc5603b9"
# subject = "cpython.test_refcounts.RefcountTestCase.test_1"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_refcounts.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_refcounts
_suite = unittest.defaultTestLoader.loadTestsFromName("RefcountTestCase.test_1", test_refcounts)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RefcountTestCase.test_1 did not pass"
print("RefcountTestCase::test_1: ok")
