# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "testresult_is_type"
# subject = "unittest.TestResult"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.TestResult: testresult_is_type (surface)."""
import unittest

assert type(unittest.TestResult).__name__ == "type"
print("testresult_is_type OK")
