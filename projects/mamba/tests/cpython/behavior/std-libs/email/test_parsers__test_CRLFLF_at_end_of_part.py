# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_parsers__test_CRLFLF_at_end_of_part"
# subject = "cpython.test_email.TestParsers.test_CRLFLF_at_end_of_part"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_email
_suite = unittest.defaultTestLoader.loadTestsFromName("TestParsers.test_CRLFLF_at_end_of_part", test_email)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestParsers.test_CRLFLF_at_end_of_part did not pass"
print("TestParsers::test_CRLFLF_at_end_of_part: ok")
