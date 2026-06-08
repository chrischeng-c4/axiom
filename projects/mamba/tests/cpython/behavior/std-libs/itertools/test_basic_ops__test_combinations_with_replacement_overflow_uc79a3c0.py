# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "test_basic_ops__test_combinations_with_replacement_overflow_uc79a3c0"
# subject = "cpython.test_itertools.TestBasicOps.test_combinations_with_replacement_overflow"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_itertools
_suite = unittest.defaultTestLoader.loadTestsFromName("TestBasicOps.test_combinations_with_replacement_overflow", test_itertools)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestBasicOps.test_combinations_with_replacement_overflow did not pass"
print("TestBasicOps::test_combinations_with_replacement_overflow: ok")
