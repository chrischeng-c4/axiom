# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "setups"
# dimension = "behavior"
# case = "test_setups__test_setup_teardown_order_with_pathological_suite_ucb29d35"
# subject = "cpython.test_setups.TestSetups.test_setup_teardown_order_with_pathological_suite"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_setups.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_setups
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSetups.test_setup_teardown_order_with_pathological_suite", test_setups)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSetups.test_setup_teardown_order_with_pathological_suite did not pass"
print("TestSetups::test_setup_teardown_order_with_pathological_suite: ok")
