# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_tests__test_or_type_operator_reference_cycle"
# subject = "cpython.test_types.UnionTests.test_or_type_operator_reference_cycle"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_types
_suite = unittest.defaultTestLoader.loadTestsFromName("UnionTests.test_or_type_operator_reference_cycle", test_types)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UnionTests.test_or_type_operator_reference_cycle did not pass"
print("UnionTests::test_or_type_operator_reference_cycle: ok")
