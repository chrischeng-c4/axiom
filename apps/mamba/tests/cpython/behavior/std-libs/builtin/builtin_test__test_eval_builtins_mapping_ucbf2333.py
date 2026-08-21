# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtin"
# dimension = "behavior"
# case = "builtin_test__test_eval_builtins_mapping_ucbf2333"
# subject = "cpython.test_builtin.BuiltinTest.test_eval_builtins_mapping"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_builtin
_suite = unittest.defaultTestLoader.loadTestsFromName("BuiltinTest.test_eval_builtins_mapping", test_builtin)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BuiltinTest.test_eval_builtins_mapping did not pass"
print("BuiltinTest::test_eval_builtins_mapping: ok")
