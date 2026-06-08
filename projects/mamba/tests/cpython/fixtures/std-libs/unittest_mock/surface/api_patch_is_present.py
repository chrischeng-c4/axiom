# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "surface"
# case = "api_patch_is_present"
# subject = "unittest.mock.patch"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.mock.patch: api_patch_is_present (surface)."""
import unittest.mock

assert hasattr(unittest.mock, "patch")
print("api_patch_is_present OK")
