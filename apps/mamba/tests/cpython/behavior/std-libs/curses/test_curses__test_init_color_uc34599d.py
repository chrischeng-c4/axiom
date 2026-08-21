# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses"
# dimension = "behavior"
# case = "test_curses__test_init_color_uc34599d"
# subject = "cpython.test_curses.TestCurses.test_init_color"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_curses.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_curses
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCurses.test_init_color", test_curses)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCurses.test_init_color did not pass"
print("TestCurses::test_init_color: ok")
