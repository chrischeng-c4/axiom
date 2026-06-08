# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "termios"
# dimension = "behavior"
# case = "test_functions__test_tcflush_uc3d8bc4"
# subject = "cpython.test_termios.TestFunctions.test_tcflush"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_termios.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_termios
_suite = unittest.defaultTestLoader.loadTestsFromName("TestFunctions.test_tcflush", test_termios)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestFunctions.test_tcflush did not pass"
print("TestFunctions::test_tcflush: ok")
