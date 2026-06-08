# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "surface"
# case = "api_call_is_present"
# subject = "unittest.mock.call"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.mock.call: api_call_is_present (surface)."""
import unittest.mock

assert hasattr(unittest.mock, "call")
print("api_call_is_present OK")
