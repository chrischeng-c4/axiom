# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_add_module_cleanup_is_present"
# subject = "unittest.addModuleCleanup"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.addModuleCleanup: api_add_module_cleanup_is_present (surface)."""
import unittest

assert hasattr(unittest, "addModuleCleanup")
print("api_add_module_cleanup_is_present OK")
