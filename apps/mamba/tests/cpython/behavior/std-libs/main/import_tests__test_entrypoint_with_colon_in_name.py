# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "main"
# dimension = "behavior"
# case = "import_tests__test_entrypoint_with_colon_in_name"
# subject = "cpython.test_main.ImportTests.test_entrypoint_with_colon_in_name"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_main.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_main
_suite = unittest.defaultTestLoader.loadTestsFromName("ImportTests.test_entrypoint_with_colon_in_name", test_main)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ImportTests.test_entrypoint_with_colon_in_name did not pass"
print("ImportTests::test_entrypoint_with_colon_in_name: ok")
