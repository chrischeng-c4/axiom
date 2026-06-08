# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "testcase_is_type"
# subject = "unittest.TestCase"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.TestCase: testcase_is_type (surface)."""
import unittest

assert type(unittest.TestCase).__name__ == "type"
print("testcase_is_type OK")
