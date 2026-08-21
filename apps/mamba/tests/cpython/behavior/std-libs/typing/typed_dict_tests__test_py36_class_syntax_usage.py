# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "typed_dict_tests__test_py36_class_syntax_usage"
# subject = "cpython.test_typing.TypedDictTests.test_py36_class_syntax_usage"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_typing
_suite = unittest.defaultTestLoader.loadTestsFromName("TypedDictTests.test_py36_class_syntax_usage", test_typing)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TypedDictTests.test_py36_class_syntax_usage did not pass"
print("TypedDictTests::test_py36_class_syntax_usage: ok")
