# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "command_line_test_case__test_option_locale_uce040df"
# subject = "cpython.test_calendar.CommandLineTestCase.test_option_locale"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_calendar
_suite = unittest.defaultTestLoader.loadTestsFromName("CommandLineTestCase.test_option_locale", test_calendar)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CommandLineTestCase.test_option_locale did not pass"
print("CommandLineTestCase::test_option_locale: ok")
