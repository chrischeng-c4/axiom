# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_global_str_is_present"
# subject = "enum.global_str"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.global_str: api_global_str_is_present (surface)."""
import enum

assert hasattr(enum, "global_str")
print("api_global_str_is_present OK")
