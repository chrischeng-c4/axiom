# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "monitoring"
# dimension = "behavior"
# case = "multiple_monitors_test__test_with_instruction_event"
# subject = "cpython.test_monitoring.MultipleMonitorsTest.test_with_instruction_event"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_monitoring.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_monitoring
_suite = unittest.defaultTestLoader.loadTestsFromName("MultipleMonitorsTest.test_with_instruction_event", test_monitoring)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MultipleMonitorsTest.test_with_instruction_event did not pass"
print("MultipleMonitorsTest::test_with_instruction_event: ok")
