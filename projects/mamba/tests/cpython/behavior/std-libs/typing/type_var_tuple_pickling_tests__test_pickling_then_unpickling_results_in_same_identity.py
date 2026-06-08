# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "type_var_tuple_pickling_tests__test_pickling_then_unpickling_results_in_same_identity"
# subject = "cpython.test_typing.TypeVarTuplePicklingTests.test_pickling_then_unpickling_results_in_same_identity"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_typing
_suite = unittest.defaultTestLoader.loadTestsFromName("TypeVarTuplePicklingTests.test_pickling_then_unpickling_results_in_same_identity", test_typing)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TypeVarTuplePicklingTests.test_pickling_then_unpickling_results_in_same_identity did not pass"
print("TypeVarTuplePicklingTests::test_pickling_then_unpickling_results_in_same_identity: ok")
