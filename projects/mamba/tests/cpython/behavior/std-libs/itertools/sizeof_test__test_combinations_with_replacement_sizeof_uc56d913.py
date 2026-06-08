# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "sizeof_test__test_combinations_with_replacement_sizeof_uc56d913"
# subject = "cpython.test_itertools.SizeofTest.test_combinations_with_replacement_sizeof"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_itertools
_suite = unittest.defaultTestLoader.loadTestsFromName("SizeofTest.test_combinations_with_replacement_sizeof", test_itertools)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SizeofTest.test_combinations_with_replacement_sizeof did not pass"
print("SizeofTest::test_combinations_with_replacement_sizeof: ok")
