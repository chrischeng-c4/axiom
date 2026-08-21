# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_params"
# dimension = "behavior"
# case = "type_params_lazy_evaluation_test__test_qualname_uc56c337"
# subject = "cpython.test_type_params.TypeParamsLazyEvaluationTest.test_qualname"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_params.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_type_params
_suite = unittest.defaultTestLoader.loadTestsFromName("TypeParamsLazyEvaluationTest.test_qualname", test_type_params)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TypeParamsLazyEvaluationTest.test_qualname did not pass"
print("TypeParamsLazyEvaluationTest::test_qualname: ok")
