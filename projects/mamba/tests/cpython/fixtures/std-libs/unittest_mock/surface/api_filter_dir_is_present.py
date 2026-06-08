# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "surface"
# case = "api_filter_dir_is_present"
# subject = "unittest.mock.FILTER_DIR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.mock.FILTER_DIR: api_filter_dir_is_present (surface)."""
import unittest.mock

assert hasattr(unittest.mock, "FILTER_DIR")
print("api_filter_dir_is_present OK")
