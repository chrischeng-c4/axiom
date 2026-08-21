# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_message_a_p_i__test_set_payload_with_non_ascii_and_charset_body_encoding_none"
# subject = "cpython.test_email.TestMessageAPI.test_set_payload_with_non_ascii_and_charset_body_encoding_none"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_email
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMessageAPI.test_set_payload_with_non_ascii_and_charset_body_encoding_none", test_email)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMessageAPI.test_set_payload_with_non_ascii_and_charset_body_encoding_none did not pass"
print("TestMessageAPI::test_set_payload_with_non_ascii_and_charset_body_encoding_none: ok")
