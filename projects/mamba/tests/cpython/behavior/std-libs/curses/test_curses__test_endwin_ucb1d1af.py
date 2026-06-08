# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses"
# dimension = "behavior"
# case = "test_curses__test_endwin_ucb1d1af"
# subject = "cpython.test_curses.TestCurses.test_endwin"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_curses.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_curses
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCurses.test_endwin", test_curses)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCurses.test_endwin did not pass"
print("TestCurses::test_endwin: ok")
