# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "windows_signal_tests__test_issue9324"
# subject = "cpython.test_signal.WindowsSignalTests.test_issue9324"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_signal
_suite = unittest.defaultTestLoader.loadTestsFromName("WindowsSignalTests.test_issue9324", test_signal)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython WindowsSignalTests.test_issue9324 did not pass"
print("WindowsSignalTests::test_issue9324: ok")
