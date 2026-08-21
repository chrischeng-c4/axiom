# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "utils"
# dimension = "behavior"
# case = "date_time_tests__test_parsedate_to_datetime_with_invalid_raises_valueerror_uc8949b5"
# subject = "cpython.test_utils.DateTimeTests.test_parsedate_to_datetime_with_invalid_raises_valueerror"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_utils.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_utils
_suite = unittest.defaultTestLoader.loadTestsFromName("DateTimeTests.test_parsedate_to_datetime_with_invalid_raises_valueerror", test_utils)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DateTimeTests.test_parsedate_to_datetime_with_invalid_raises_valueerror did not pass"
print("DateTimeTests::test_parsedate_to_datetime_with_invalid_raises_valueerror: ok")
