# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "pidfd_signal_test__test_pidfd_send_signal"
# subject = "cpython.test_signal.PidfdSignalTest.test_pidfd_send_signal"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_signal
_suite = unittest.defaultTestLoader.loadTestsFromName("PidfdSignalTest.test_pidfd_send_signal", test_signal)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PidfdSignalTest.test_pidfd_send_signal did not pass"
print("PidfdSignalTest::test_pidfd_send_signal: ok")
