# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "policy"
# dimension = "behavior"
# case = "test_policy_propagation__test_parser_propagates_policy_to_sub_messages_uc11652d"
# subject = "cpython.test_policy.TestPolicyPropagation.test_parser_propagates_policy_to_sub_messages"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_policy.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_policy
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPolicyPropagation.test_parser_propagates_policy_to_sub_messages", test_policy)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPolicyPropagation.test_parser_propagates_policy_to_sub_messages did not pass"
print("TestPolicyPropagation::test_parser_propagates_policy_to_sub_messages: ok")
