# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "test_get_terminal_size__test_stty_match"
# subject = "cpython.test_shutil.TestGetTerminalSize.test_stty_match"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_shutil
_suite = unittest.defaultTestLoader.loadTestsFromName("TestGetTerminalSize.test_stty_match", test_shutil)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestGetTerminalSize.test_stty_match did not pass"
print("TestGetTerminalSize::test_stty_match: ok")
