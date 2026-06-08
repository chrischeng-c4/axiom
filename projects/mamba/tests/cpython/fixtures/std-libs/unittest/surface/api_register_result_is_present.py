# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_register_result_is_present"
# subject = "unittest.registerResult"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.registerResult: api_register_result_is_present (surface)."""
import unittest

assert hasattr(unittest, "registerResult")
print("api_register_result_is_present OK")
