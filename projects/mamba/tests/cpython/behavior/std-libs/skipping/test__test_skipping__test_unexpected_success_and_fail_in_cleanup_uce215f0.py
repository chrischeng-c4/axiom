# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "skipping"
# dimension = "behavior"
# case = "test__test_skipping__test_unexpected_success_and_fail_in_cleanup_uce215f0"
# subject = "cpython.test_skipping.Test_TestSkipping.test_unexpected_success_and_fail_in_cleanup"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_skipping.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_skipping
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_TestSkipping.test_unexpected_success_and_fail_in_cleanup", test_skipping)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_TestSkipping.test_unexpected_success_and_fail_in_cleanup did not pass"
print("Test_TestSkipping::test_unexpected_success_and_fail_in_cleanup: ok")
