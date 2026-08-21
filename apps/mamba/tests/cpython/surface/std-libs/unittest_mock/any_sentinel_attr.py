# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "surface"
# case = "any_sentinel_attr"
# subject = "unittest.mock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.mock: any_sentinel_attr (surface)."""
import unittest.mock

assert hasattr(unittest.mock, "ANY")
print("any_sentinel_attr OK")
