# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "surface"
# case = "api_copy_is_present"
# subject = "copy.copy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""copy.copy: api_copy_is_present (surface)."""
import copy

assert hasattr(copy, "copy")
print("api_copy_is_present OK")
