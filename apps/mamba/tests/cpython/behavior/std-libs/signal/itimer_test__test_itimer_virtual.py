# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "itimer_test__test_itimer_virtual"
# subject = "cpython.test_signal.ItimerTest.test_itimer_virtual"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_signal
_suite = unittest.defaultTestLoader.loadTestsFromName("ItimerTest.test_itimer_virtual", test_signal)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ItimerTest.test_itimer_virtual did not pass"
print("ItimerTest::test_itimer_virtual: ok")
