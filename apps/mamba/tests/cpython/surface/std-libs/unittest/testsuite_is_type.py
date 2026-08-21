# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "testsuite_is_type"
# subject = "unittest.TestSuite"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.TestSuite: testsuite_is_type (surface)."""
import unittest

assert type(unittest.TestSuite).__name__ == "type"
print("testsuite_is_type OK")
