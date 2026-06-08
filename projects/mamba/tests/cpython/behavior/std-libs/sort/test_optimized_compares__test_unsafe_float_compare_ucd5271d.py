# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sort"
# dimension = "behavior"
# case = "test_optimized_compares__test_unsafe_float_compare_ucd5271d"
# subject = "cpython.test_sort.TestOptimizedCompares.test_unsafe_float_compare"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sort.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sort
_suite = unittest.defaultTestLoader.loadTestsFromName("TestOptimizedCompares.test_unsafe_float_compare", test_sort)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestOptimizedCompares.test_unsafe_float_compare did not pass"
print("TestOptimizedCompares::test_unsafe_float_compare: ok")
