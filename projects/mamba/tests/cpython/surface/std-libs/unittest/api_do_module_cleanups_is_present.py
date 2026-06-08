# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_do_module_cleanups_is_present"
# subject = "unittest.doModuleCleanups"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.doModuleCleanups: api_do_module_cleanups_is_present (surface)."""
import unittest

assert hasattr(unittest, "doModuleCleanups")
print("api_do_module_cleanups_is_present OK")
