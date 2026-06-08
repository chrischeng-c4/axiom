# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_path_hooks_is_present"
# subject = "sys.path_hooks"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.path_hooks: api_path_hooks_is_present (surface)."""
import sys

assert hasattr(sys, "path_hooks")
print("api_path_hooks_is_present OK")
