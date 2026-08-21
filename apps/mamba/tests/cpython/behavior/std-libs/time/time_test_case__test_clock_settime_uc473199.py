# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "time_test_case__test_clock_settime_uc473199"
# subject = "cpython.test_time.TimeTestCase.test_clock_settime"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_time
_suite = unittest.defaultTestLoader.loadTestsFromName("TimeTestCase.test_clock_settime", test_time)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TimeTestCase.test_clock_settime did not pass"
print("TimeTestCase::test_clock_settime: ok")
