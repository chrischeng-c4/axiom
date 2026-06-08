# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_enter_module_context_is_present"
# subject = "unittest.enterModuleContext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.enterModuleContext: api_enter_module_context_is_present (surface)."""
import unittest

assert hasattr(unittest, "enterModuleContext")
print("api_enter_module_context_is_present OK")
