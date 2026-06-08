# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtin"
# dimension = "behavior"
# case = "builtin_test__test_all_uc154f3d"
# subject = "cpython.test_builtin.BuiltinTest.test_all"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_builtin
_suite = unittest.defaultTestLoader.loadTestsFromName("BuiltinTest.test_all", test_builtin)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BuiltinTest.test_all did not pass"
print("BuiltinTest::test_all: ok")
