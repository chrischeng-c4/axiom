# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code_module"
# dimension = "behavior"
# case = "test_interactive_console__test_ps1_ucb35d19"
# subject = "cpython.test_code_module.TestInteractiveConsole.test_ps1"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_code_module.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_code_module
_suite = unittest.defaultTestLoader.loadTestsFromName("TestInteractiveConsole.test_ps1", test_code_module)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestInteractiveConsole.test_ps1 did not pass"
print("TestInteractiveConsole::test_ps1: ok")
