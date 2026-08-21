# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_params"
# dimension = "behavior"
# case = "type_params_invalid_test__test_disallowed_expressions_ucf28ee4"
# subject = "cpython.test_type_params.TypeParamsInvalidTest.test_disallowed_expressions"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_params.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_type_params
_suite = unittest.defaultTestLoader.loadTestsFromName("TypeParamsInvalidTest.test_disallowed_expressions", test_type_params)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TypeParamsInvalidTest.test_disallowed_expressions did not pass"
print("TypeParamsInvalidTest::test_disallowed_expressions: ok")
