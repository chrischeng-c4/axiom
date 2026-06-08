# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "testresult_initial_counters_empty"
# subject = "unittest.TestResult"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestResult: a fresh TestResult starts with empty failures and errors lists"""
import unittest

result = unittest.TestResult()
assert result.failures == []
assert result.errors == []
print("testresult_initial_counters_empty OK")
