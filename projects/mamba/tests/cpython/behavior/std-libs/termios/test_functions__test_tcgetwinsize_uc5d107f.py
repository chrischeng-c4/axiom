# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "termios"
# dimension = "behavior"
# case = "test_functions__test_tcgetwinsize_uc5d107f"
# subject = "cpython.test_termios.TestFunctions.test_tcgetwinsize"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_termios.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_termios
_suite = unittest.defaultTestLoader.loadTestsFromName("TestFunctions.test_tcgetwinsize", test_termios)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestFunctions.test_tcgetwinsize did not pass"
print("TestFunctions::test_tcgetwinsize: ok")
