# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "locale_time__tests__test_timezone_uc80ef61"
# subject = "cpython.test_strptime.LocaleTime_Tests.test_timezone"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_strptime
_suite = unittest.defaultTestLoader.loadTestsFromName("LocaleTime_Tests.test_timezone", test_strptime)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LocaleTime_Tests.test_timezone did not pass"
print("LocaleTime_Tests::test_timezone: ok")
