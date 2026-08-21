# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "testcase_assertequal_is_callable"
# subject = "unittest.TestCase.assertEqual"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.TestCase.assertEqual: testcase_assertequal_is_callable (surface)."""
import unittest

assert callable(unittest.TestCase.assertEqual)
print("testcase_assertequal_is_callable OK")
