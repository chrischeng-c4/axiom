# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "setups"
# dimension = "behavior"
# case = "test_setups__test_class_not_setup_or_torndown_when_skipped_uc0e5db3"
# subject = "cpython.test_setups.TestSetups.test_class_not_setup_or_torndown_when_skipped"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_setups.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_setups
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSetups.test_class_not_setup_or_torndown_when_skipped", test_setups)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSetups.test_class_not_setup_or_torndown_when_skipped did not pass"
print("TestSetups::test_class_not_setup_or_torndown_when_skipped: ok")
