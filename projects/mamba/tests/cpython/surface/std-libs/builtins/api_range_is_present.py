# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_range_is_present"
# subject = "builtins.range"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.range: api_range_is_present (surface)."""
import builtins

assert hasattr(builtins, "range")
print("api_range_is_present OK")
