# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "strptime_tests__test_date_time_locale2_uc1b8681"
# subject = "cpython.test_strptime.StrptimeTests.test_date_time_locale2"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_strptime
_suite = unittest.defaultTestLoader.loadTestsFromName("StrptimeTests.test_date_time_locale2", test_strptime)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StrptimeTests.test_date_time_locale2 did not pass"
print("StrptimeTests::test_date_time_locale2: ok")
