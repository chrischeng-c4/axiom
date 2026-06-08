# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "eval"
# dimension = "behavior"
# case = "tests__test_eval_set_recursion_limit_uc94bd47"
# subject = "cpython.test_eval.Tests.test_eval_set_recursion_limit"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_eval.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_eval
_suite = unittest.defaultTestLoader.loadTestsFromName("Tests.test_eval_set_recursion_limit", test_eval)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Tests.test_eval_set_recursion_limit did not pass"
print("Tests::test_eval_set_recursion_limit: ok")
