# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "sizeof_test__test_permutations_sizeof_uc04ad20"
# subject = "cpython.test_itertools.SizeofTest.test_permutations_sizeof"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_itertools
_suite = unittest.defaultTestLoader.loadTestsFromName("SizeofTest.test_permutations_sizeof", test_itertools)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SizeofTest.test_permutations_sizeof did not pass"
print("SizeofTest::test_permutations_sizeof: ok")
