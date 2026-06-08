# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "utils"
# dimension = "behavior"
# case = "localtime_tests__test_localtime_daylight_false_dst_true_uccdf827"
# subject = "cpython.test_utils.LocaltimeTests.test_localtime_daylight_false_dst_true"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_utils.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_utils
_suite = unittest.defaultTestLoader.loadTestsFromName("LocaltimeTests.test_localtime_daylight_false_dst_true", test_utils)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LocaltimeTests.test_localtime_daylight_false_dst_true did not pass"
print("LocaltimeTests::test_localtime_daylight_false_dst_true: ok")
