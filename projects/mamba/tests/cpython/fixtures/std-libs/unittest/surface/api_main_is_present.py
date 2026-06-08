# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_main_is_present"
# subject = "unittest.main"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.main: api_main_is_present (surface)."""
import unittest.main

assert hasattr(unittest, "main")
print("api_main_is_present OK")
