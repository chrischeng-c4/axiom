# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "program"
# dimension = "behavior"
# case = "foo_bar__testexpectedfailure_uc4dd75c"
# subject = "cpython.test_program.Test_TestProgram.FooBar.testExpectedFailure"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_program.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_program
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_TestProgram.FooBar.testExpectedFailure", test_program)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_TestProgram.FooBar.testExpectedFailure did not pass"
print("Test_TestProgram.FooBar::testExpectedFailure: ok")
