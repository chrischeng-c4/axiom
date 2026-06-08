# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_stdlib_module_names_is_present"
# subject = "sys.stdlib_module_names"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.stdlib_module_names: api_stdlib_module_names_is_present (surface)."""
import sys

assert hasattr(sys, "stdlib_module_names")
print("api_stdlib_module_names_is_present OK")
