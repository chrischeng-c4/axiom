# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "policy"
# dimension = "behavior"
# case = "test_policy_propagation__test_message_policy_used_by_as_string_uc7fa372"
# subject = "cpython.test_policy.TestPolicyPropagation.test_message_policy_used_by_as_string"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_policy.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_policy
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPolicyPropagation.test_message_policy_used_by_as_string", test_policy)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPolicyPropagation.test_message_policy_used_by_as_string did not pass"
print("TestPolicyPropagation::test_message_policy_used_by_as_string: ok")
