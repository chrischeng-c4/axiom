# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "typed_dict_tests__test_typeddict_special_keyword_names"
# subject = "cpython.test_typing.TypedDictTests.test_typeddict_special_keyword_names"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_typing
_suite = unittest.defaultTestLoader.loadTestsFromName("TypedDictTests.test_typeddict_special_keyword_names", test_typing)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TypedDictTests.test_typeddict_special_keyword_names did not pass"
print("TypedDictTests::test_typeddict_special_keyword_names: ok")
