# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "testcase_run_is_callable"
# subject = "unittest.TestCase.run"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.TestCase.run: testcase_run_is_callable (surface)."""
import unittest

assert callable(unittest.TestCase.run)
print("testcase_run_is_callable OK")
