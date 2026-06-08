# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_default_test_loader_is_present"
# subject = "unittest.defaultTestLoader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.defaultTestLoader: api_default_test_loader_is_present (surface)."""
import unittest

assert hasattr(unittest, "defaultTestLoader")
print("api_default_test_loader_is_present OK")
