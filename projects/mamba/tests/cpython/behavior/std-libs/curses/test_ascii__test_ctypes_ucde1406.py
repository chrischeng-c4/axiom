# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses"
# dimension = "behavior"
# case = "test_ascii__test_ctypes_ucde1406"
# subject = "cpython.test_curses.TestAscii.test_ctypes"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_curses.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_curses
_suite = unittest.defaultTestLoader.loadTestsFromName("TestAscii.test_ctypes", test_curses)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestAscii.test_ctypes did not pass"
print("TestAscii::test_ctypes: ok")
