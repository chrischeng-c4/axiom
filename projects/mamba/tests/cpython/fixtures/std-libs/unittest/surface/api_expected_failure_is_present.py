# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_expected_failure_is_present"
# subject = "unittest.expectedFailure"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.expectedFailure: api_expected_failure_is_present (surface)."""
import unittest

assert hasattr(unittest, "expectedFailure")
print("api_expected_failure_is_present OK")
