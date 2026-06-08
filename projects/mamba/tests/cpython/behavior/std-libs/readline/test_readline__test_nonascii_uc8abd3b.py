# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "behavior"
# case = "test_readline__test_nonascii_uc8abd3b"
# subject = "cpython.test_readline.TestReadline.test_nonascii"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_readline.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_readline
_suite = unittest.defaultTestLoader.loadTestsFromName("TestReadline.test_nonascii", test_readline)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestReadline.test_nonascii did not pass"
print("TestReadline::test_nonascii: ok")
