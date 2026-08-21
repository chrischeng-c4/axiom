# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "get_type_hint_tests__test_get_type_hints_modules_forwardref"
# subject = "cpython.test_typing.GetTypeHintTests.test_get_type_hints_modules_forwardref"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_typing
_suite = unittest.defaultTestLoader.loadTestsFromName("GetTypeHintTests.test_get_type_hints_modules_forwardref", test_typing)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GetTypeHintTests.test_get_type_hints_modules_forwardref did not pass"
print("GetTypeHintTests::test_get_type_hints_modules_forwardref: ok")
