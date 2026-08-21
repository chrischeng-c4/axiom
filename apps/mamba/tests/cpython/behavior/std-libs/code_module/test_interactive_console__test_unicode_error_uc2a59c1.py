# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code_module"
# dimension = "behavior"
# case = "test_interactive_console__test_unicode_error_uc2a59c1"
# subject = "cpython.test_code_module.TestInteractiveConsole.test_unicode_error"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_code_module.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_code_module
_suite = unittest.defaultTestLoader.loadTestsFromName("TestInteractiveConsole.test_unicode_error", test_code_module)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestInteractiveConsole.test_unicode_error did not pass"
print("TestInteractiveConsole::test_unicode_error: ok")
