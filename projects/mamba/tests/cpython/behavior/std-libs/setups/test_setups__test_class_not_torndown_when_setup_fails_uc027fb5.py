# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "setups"
# dimension = "behavior"
# case = "test_setups__test_class_not_torndown_when_setup_fails_uc027fb5"
# subject = "cpython.test_setups.TestSetups.test_class_not_torndown_when_setup_fails"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_setups.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_setups
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSetups.test_class_not_torndown_when_setup_fails", test_setups)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSetups.test_class_not_torndown_when_setup_fails did not pass"
print("TestSetups::test_class_not_torndown_when_setup_fails: ok")
