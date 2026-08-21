# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "program"
# dimension = "behavior"
# case = "test__test_program__test_defaulttest_with_string_uc77d410"
# subject = "cpython.test_program.Test_TestProgram.test_defaultTest_with_string"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_program.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_program
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_TestProgram.test_defaultTest_with_string", test_program)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_TestProgram.test_defaultTest_with_string did not pass"
print("Test_TestProgram::test_defaultTest_with_string: ok")
