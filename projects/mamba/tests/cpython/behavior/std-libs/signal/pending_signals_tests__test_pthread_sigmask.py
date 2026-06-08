# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "pending_signals_tests__test_pthread_sigmask"
# subject = "cpython.test_signal.PendingSignalsTests.test_pthread_sigmask"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_signal
_suite = unittest.defaultTestLoader.loadTestsFromName("PendingSignalsTests.test_pthread_sigmask", test_signal)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PendingSignalsTests.test_pthread_sigmask did not pass"
print("PendingSignalsTests::test_pthread_sigmask: ok")
