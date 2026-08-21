# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "break"
# dimension = "behavior"
# case = "test_break__testinstallhandler_ucd68f7d"
# subject = "cpython.test_break.TestBreak.testInstallHandler"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_break.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_break
_suite = unittest.defaultTestLoader.loadTestsFromName("TestBreak.testInstallHandler", test_break)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestBreak.testInstallHandler did not pass"
print("TestBreak::testInstallHandler: ok")
