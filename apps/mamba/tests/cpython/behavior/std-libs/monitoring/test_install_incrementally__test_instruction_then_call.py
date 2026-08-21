# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "monitoring"
# dimension = "behavior"
# case = "test_install_incrementally__test_instruction_then_call"
# subject = "cpython.test_monitoring.TestInstallIncrementally.test_instruction_then_call"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_monitoring.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_monitoring
_suite = unittest.defaultTestLoader.loadTestsFromName("TestInstallIncrementally.test_instruction_then_call", test_monitoring)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestInstallIncrementally.test_instruction_then_call did not pass"
print("TestInstallIncrementally::test_instruction_then_call: ok")
