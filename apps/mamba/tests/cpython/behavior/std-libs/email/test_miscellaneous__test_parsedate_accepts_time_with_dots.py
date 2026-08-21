# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_miscellaneous__test_parsedate_accepts_time_with_dots"
# subject = "cpython.test_email.TestMiscellaneous.test_parsedate_accepts_time_with_dots"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_email
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMiscellaneous.test_parsedate_accepts_time_with_dots", test_email)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMiscellaneous.test_parsedate_accepts_time_with_dots did not pass"
print("TestMiscellaneous::test_parsedate_accepts_time_with_dots: ok")
