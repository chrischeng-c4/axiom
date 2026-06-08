# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "wakeup_socket_signal_tests__test_warn_on_full_buffer"
# subject = "cpython.test_signal.WakeupSocketSignalTests.test_warn_on_full_buffer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_signal
_suite = unittest.defaultTestLoader.loadTestsFromName("WakeupSocketSignalTests.test_warn_on_full_buffer", test_signal)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython WakeupSocketSignalTests.test_warn_on_full_buffer did not pass"
print("WakeupSocketSignalTests::test_warn_on_full_buffer: ok")
