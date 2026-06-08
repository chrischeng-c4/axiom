# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "program"
# dimension = "behavior"
# case = "test_command_line_args__testparseargsnonexistentfiles_ucdd5ca9"
# subject = "cpython.test_program.TestCommandLineArgs.testParseArgsNonExistentFiles"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_program.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_program
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCommandLineArgs.testParseArgsNonExistentFiles", test_program)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCommandLineArgs.testParseArgsNonExistentFiles did not pass"
print("TestCommandLineArgs::testParseArgsNonExistentFiles: ok")
