# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "refcounts"
# dimension = "behavior"
# case = "refcount_test_case__test_refcount_ucd2a0cb"
# subject = "cpython.test_refcounts.RefcountTestCase.test_refcount"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_refcounts.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_refcounts
_suite = unittest.defaultTestLoader.loadTestsFromName("RefcountTestCase.test_refcount", test_refcounts)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RefcountTestCase.test_refcount did not pass"
print("RefcountTestCase::test_refcount: ok")
