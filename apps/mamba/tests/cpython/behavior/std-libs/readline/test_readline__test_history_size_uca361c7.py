# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "behavior"
# case = "test_readline__test_history_size_uca361c7"
# subject = "cpython.test_readline.TestReadline.test_history_size"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_readline.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_readline
_suite = unittest.defaultTestLoader.loadTestsFromName("TestReadline.test_history_size", test_readline)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestReadline.test_history_size did not pass"
print("TestReadline::test_history_size: ok")
