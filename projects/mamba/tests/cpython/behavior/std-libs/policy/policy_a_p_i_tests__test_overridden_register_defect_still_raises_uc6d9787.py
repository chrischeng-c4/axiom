# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "policy"
# dimension = "behavior"
# case = "policy_a_p_i_tests__test_overridden_register_defect_still_raises_uc6d9787"
# subject = "cpython.test_policy.PolicyAPITests.test_overridden_register_defect_still_raises"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_policy.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_policy
_suite = unittest.defaultTestLoader.loadTestsFromName("PolicyAPITests.test_overridden_register_defect_still_raises", test_policy)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PolicyAPITests.test_overridden_register_defect_still_raises did not pass"
print("PolicyAPITests::test_overridden_register_defect_still_raises: ok")
