# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_signed__test_long_headers_as_string_maxheaderlen"
# subject = "cpython.test_email.TestSigned.test_long_headers_as_string_maxheaderlen"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_email
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSigned.test_long_headers_as_string_maxheaderlen", test_email)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSigned.test_long_headers_as_string_maxheaderlen did not pass"
print("TestSigned::test_long_headers_as_string_maxheaderlen: ok")
