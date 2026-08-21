# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "policy"
# dimension = "behavior"
# case = "policy_a_p_i_tests__test_defaults_ucac7c1b"
# subject = "cpython.test_policy.PolicyAPITests.test_defaults"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_policy.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_policy
_suite = unittest.defaultTestLoader.loadTestsFromName("PolicyAPITests.test_defaults", test_policy)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PolicyAPITests.test_defaults did not pass"
print("PolicyAPITests::test_defaults: ok")
