# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "test_c_a_p_i__test_untrack_uce0d5f9"
# subject = "cpython.test_tracemalloc.TestCAPI.test_untrack"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tracemalloc
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCAPI.test_untrack", test_tracemalloc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCAPI.test_untrack did not pass"
print("TestCAPI::test_untrack: ok")
