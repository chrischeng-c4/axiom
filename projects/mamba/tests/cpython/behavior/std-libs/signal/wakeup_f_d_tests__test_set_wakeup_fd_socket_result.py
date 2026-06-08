# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "wakeup_f_d_tests__test_set_wakeup_fd_socket_result"
# subject = "cpython.test_signal.WakeupFDTests.test_set_wakeup_fd_socket_result"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_signal
_suite = unittest.defaultTestLoader.loadTestsFromName("WakeupFDTests.test_set_wakeup_fd_socket_result", test_signal)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython WakeupFDTests.test_set_wakeup_fd_socket_result did not pass"
print("WakeupFDTests::test_set_wakeup_fd_socket_result: ok")
