# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "all_tests__test_all_exported_names"
# subject = "cpython.test_typing.AllTests.test_all_exported_names"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_typing
_suite = unittest.defaultTestLoader.loadTestsFromName("AllTests.test_all_exported_names", test_typing)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AllTests.test_all_exported_names did not pass"
print("AllTests::test_all_exported_names: ok")
