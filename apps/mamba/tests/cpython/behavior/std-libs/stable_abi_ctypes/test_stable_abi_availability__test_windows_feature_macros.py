# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stable_abi_ctypes"
# dimension = "behavior"
# case = "test_stable_abi_availability__test_windows_feature_macros"
# subject = "cpython.test_stable_abi_ctypes.TestStableABIAvailability.test_windows_feature_macros"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_stable_abi_ctypes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: TestStableABIAvailability::test_windows_feature_macros (CPython 3.12 oracle)."""

import sys
import unittest
from test.test_stable_abi_ctypes import TestStableABIAvailability


case = TestStableABIAvailability("test_windows_feature_macros")
result = unittest.TestResult()
case.run(result)

assert result.wasSuccessful(), result
if sys.platform != "win32":
    assert len(result.skipped) == 1, result.skipped
    assert result.skipped[0][1] == "Windows specific test"
else:
    assert not result.skipped, result.skipped
assert not result.failures, result.failures
assert not result.errors, result.errors

print("TestStableABIAvailability::test_windows_feature_macros: ok")
