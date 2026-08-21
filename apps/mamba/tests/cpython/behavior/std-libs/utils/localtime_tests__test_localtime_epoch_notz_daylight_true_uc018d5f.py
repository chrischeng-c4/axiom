# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "utils"
# dimension = "behavior"
# case = "localtime_tests__test_localtime_epoch_notz_daylight_true_uc018d5f"
# subject = "cpython.test_utils.LocaltimeTests.test_localtime_epoch_notz_daylight_true"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_utils.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_utils
_suite = unittest.defaultTestLoader.loadTestsFromName("LocaltimeTests.test_localtime_epoch_notz_daylight_true", test_utils)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LocaltimeTests.test_localtime_epoch_notz_daylight_true did not pass"
print("LocaltimeTests::test_localtime_epoch_notz_daylight_true: ok")
