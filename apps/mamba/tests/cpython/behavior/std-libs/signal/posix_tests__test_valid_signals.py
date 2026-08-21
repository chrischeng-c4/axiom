# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "posix_tests__test_valid_signals"
# subject = "cpython.test_signal.PosixTests.test_valid_signals"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_signal
_suite = unittest.defaultTestLoader.loadTestsFromName("PosixTests.test_valid_signals", test_signal)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PosixTests.test_valid_signals did not pass"
print("PosixTests::test_valid_signals: ok")
