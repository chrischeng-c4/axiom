# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "surface"
# case = "api_error_is_present"
# subject = "copy.Error"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""copy.Error: api_error_is_present (surface)."""
import copy

assert hasattr(copy, "Error")
print("api_error_is_present OK")
