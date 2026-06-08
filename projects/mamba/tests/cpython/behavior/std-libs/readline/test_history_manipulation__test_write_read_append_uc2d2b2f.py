# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "behavior"
# case = "test_history_manipulation__test_write_read_append_uc2d2b2f"
# subject = "cpython.test_readline.TestHistoryManipulation.test_write_read_append"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_readline.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_readline
_suite = unittest.defaultTestLoader.loadTestsFromName("TestHistoryManipulation.test_write_read_append", test_readline)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestHistoryManipulation.test_write_read_append did not pass"
print("TestHistoryManipulation::test_write_read_append: ok")
