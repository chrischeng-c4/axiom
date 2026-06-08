# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_modules_is_present"
# subject = "sys.modules"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.modules: api_modules_is_present (surface)."""
import sys

assert hasattr(sys, "modules")
print("api_modules_is_present OK")
