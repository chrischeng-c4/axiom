# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "program"
# dimension = "behavior"
# case = "test__test_program__testnoexit_uc71edbc"
# subject = "cpython.test_program.Test_TestProgram.testNoExit"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_program.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_program
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_TestProgram.testNoExit", test_program)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_TestProgram.testNoExit did not pass"
print("Test_TestProgram::testNoExit: ok")
