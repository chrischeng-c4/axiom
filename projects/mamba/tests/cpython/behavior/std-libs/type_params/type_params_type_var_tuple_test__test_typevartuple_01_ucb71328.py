# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_params"
# dimension = "behavior"
# case = "type_params_type_var_tuple_test__test_typevartuple_01_ucb71328"
# subject = "cpython.test_type_params.TypeParamsTypeVarTupleTest.test_typevartuple_01"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_params.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_type_params
_suite = unittest.defaultTestLoader.loadTestsFromName("TypeParamsTypeVarTupleTest.test_typevartuple_01", test_type_params)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TypeParamsTypeVarTupleTest.test_typevartuple_01 did not pass"
print("TypeParamsTypeVarTupleTest::test_typevartuple_01: ok")
