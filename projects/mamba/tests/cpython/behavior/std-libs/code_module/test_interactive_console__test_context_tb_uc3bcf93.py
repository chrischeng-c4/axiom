# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code_module"
# dimension = "behavior"
# case = "test_interactive_console__test_context_tb_uc3bcf93"
# subject = "cpython.test_code_module.TestInteractiveConsole.test_context_tb"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_code_module.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_code_module
_suite = unittest.defaultTestLoader.loadTestsFromName("TestInteractiveConsole.test_context_tb", test_code_module)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestInteractiveConsole.test_context_tb did not pass"
print("TestInteractiveConsole::test_context_tb: ok")
