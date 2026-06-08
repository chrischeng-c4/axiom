# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_r_f_c2231__test_should_not_hang_on_invalid_ew_messages"
# subject = "cpython.test_email.TestRFC2231.test_should_not_hang_on_invalid_ew_messages"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_email
_suite = unittest.defaultTestLoader.loadTestsFromName("TestRFC2231.test_should_not_hang_on_invalid_ew_messages", test_email)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestRFC2231.test_should_not_hang_on_invalid_ew_messages did not pass"
print("TestRFC2231::test_should_not_hang_on_invalid_ew_messages: ok")
