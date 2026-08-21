# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "wakeup_signal_tests__test_pending"
# subject = "cpython.test_signal.WakeupSignalTests.test_pending"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_signal
_suite = unittest.defaultTestLoader.loadTestsFromName("WakeupSignalTests.test_pending", test_signal)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython WakeupSignalTests.test_pending did not pass"
print("WakeupSignalTests::test_pending: ok")
