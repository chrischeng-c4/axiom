# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_remove_handler_is_present"
# subject = "unittest.removeHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.removeHandler: api_remove_handler_is_present (surface)."""
import unittest

assert hasattr(unittest, "removeHandler")
print("api_remove_handler_is_present OK")
