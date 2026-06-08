# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_header__test_encode_preserves_leading_ws_on_value"
# subject = "cpython.test_email.TestHeader.test_encode_preserves_leading_ws_on_value"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_email
_suite = unittest.defaultTestLoader.loadTestsFromName("TestHeader.test_encode_preserves_leading_ws_on_value", test_email)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestHeader.test_encode_preserves_leading_ws_on_value did not pass"
print("TestHeader::test_encode_preserves_leading_ws_on_value: ok")
