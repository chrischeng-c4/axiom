# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "test_classes_and_functions__test_getfullargspec_builtin_methods"
# subject = "cpython.test_inspect.TestClassesAndFunctions.test_getfullargspec_builtin_methods"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_inspect/test_inspect.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_inspect import test_inspect
_suite = unittest.defaultTestLoader.loadTestsFromName("TestClassesAndFunctions.test_getfullargspec_builtin_methods", test_inspect)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestClassesAndFunctions.test_getfullargspec_builtin_methods did not pass"
print("TestClassesAndFunctions::test_getfullargspec_builtin_methods: ok")
