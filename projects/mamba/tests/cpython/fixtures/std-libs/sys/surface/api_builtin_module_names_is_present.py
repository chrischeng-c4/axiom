# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_builtin_module_names_is_present"
# subject = "sys.builtin_module_names"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.builtin_module_names: api_builtin_module_names_is_present (surface)."""
import sys

assert hasattr(sys, "builtin_module_names")
print("api_builtin_module_names_is_present OK")
