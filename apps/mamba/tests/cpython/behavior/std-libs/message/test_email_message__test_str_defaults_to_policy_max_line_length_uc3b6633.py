# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "message"
# dimension = "behavior"
# case = "test_email_message__test_str_defaults_to_policy_max_line_length_uc3b6633"
# subject = "cpython.test_message.TestEmailMessage.test_str_defaults_to_policy_max_line_length"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_message.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_message
_suite = unittest.defaultTestLoader.loadTestsFromName("TestEmailMessage.test_str_defaults_to_policy_max_line_length", test_message)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestEmailMessage.test_str_defaults_to_policy_max_line_length did not pass"
print("TestEmailMessage::test_str_defaults_to_policy_max_line_length: ok")
