# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "program"
# dimension = "behavior"
# case = "test_command_line_args__testcatchbreakinstallshandler_uc08cd30"
# subject = "cpython.test_program.TestCommandLineArgs.testCatchBreakInstallsHandler"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_program.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_program
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCommandLineArgs.testCatchBreakInstallsHandler", test_program)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCommandLineArgs.testCatchBreakInstallsHandler did not pass"
print("TestCommandLineArgs::testCatchBreakInstallsHandler: ok")
