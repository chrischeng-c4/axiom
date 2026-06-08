# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "calendar_test_case__test_deprecation_warning_ucbdfde4"
# subject = "cpython.test_calendar.CalendarTestCase.test_deprecation_warning"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_calendar
_suite = unittest.defaultTestLoader.loadTestsFromName("CalendarTestCase.test_deprecation_warning", test_calendar)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CalendarTestCase.test_deprecation_warning did not pass"
print("CalendarTestCase::test_deprecation_warning: ok")
