# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "eval_code_ex"
# dimension = "behavior"
# case = "py_eval__eval_code_ex_tests__test_with_default_uc22a08d"
# subject = "cpython.test_eval_code_ex.PyEval_EvalCodeExTests.test_with_default"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_eval_code_ex.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_eval_code_ex
_suite = unittest.defaultTestLoader.loadTestsFromName("PyEval_EvalCodeExTests.test_with_default", test_eval_code_ex)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PyEval_EvalCodeExTests.test_with_default did not pass"
print("PyEval_EvalCodeExTests::test_with_default: ok")
