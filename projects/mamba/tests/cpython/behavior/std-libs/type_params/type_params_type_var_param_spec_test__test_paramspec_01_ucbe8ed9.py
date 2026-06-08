# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_params"
# dimension = "behavior"
# case = "type_params_type_var_param_spec_test__test_paramspec_01_ucbe8ed9"
# subject = "cpython.test_type_params.TypeParamsTypeVarParamSpecTest.test_paramspec_01"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_params.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_type_params
_suite = unittest.defaultTestLoader.loadTestsFromName("TypeParamsTypeVarParamSpecTest.test_paramspec_01", test_type_params)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TypeParamsTypeVarParamSpecTest.test_paramspec_01 did not pass"
print("TypeParamsTypeVarParamSpecTest::test_paramspec_01: ok")
