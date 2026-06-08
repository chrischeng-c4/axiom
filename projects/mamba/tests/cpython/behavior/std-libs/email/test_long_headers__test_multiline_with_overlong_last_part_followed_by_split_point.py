# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_long_headers__test_multiline_with_overlong_last_part_followed_by_split_point"
# subject = "cpython.test_email.TestLongHeaders.test_multiline_with_overlong_last_part_followed_by_split_point"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_email
_suite = unittest.defaultTestLoader.loadTestsFromName("TestLongHeaders.test_multiline_with_overlong_last_part_followed_by_split_point", test_email)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestLongHeaders.test_multiline_with_overlong_last_part_followed_by_split_point did not pass"
print("TestLongHeaders::test_multiline_with_overlong_last_part_followed_by_split_point: ok")
