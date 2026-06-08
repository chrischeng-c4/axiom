# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "parser"
# dimension = "behavior"
# case = "test_custom_message__test_custom_message_gets_policy_if_possible_from_string"
# subject = "cpython.test_parser.TestCustomMessage.test_custom_message_gets_policy_if_possible_from_string"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_parser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_parser
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCustomMessage.test_custom_message_gets_policy_if_possible_from_string", test_parser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCustomMessage.test_custom_message_gets_policy_if_possible_from_string did not pass"
print("TestCustomMessage::test_custom_message_gets_policy_if_possible_from_string: ok")
