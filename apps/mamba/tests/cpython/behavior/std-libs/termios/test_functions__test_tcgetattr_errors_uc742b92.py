# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "termios"
# dimension = "behavior"
# case = "test_functions__test_tcgetattr_errors_uc742b92"
# subject = "cpython.test_termios.TestFunctions.test_tcgetattr_errors"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_termios.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_termios
_suite = unittest.defaultTestLoader.loadTestsFromName("TestFunctions.test_tcgetattr_errors", test_termios)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestFunctions.test_tcgetattr_errors did not pass"
print("TestFunctions::test_tcgetattr_errors: ok")
