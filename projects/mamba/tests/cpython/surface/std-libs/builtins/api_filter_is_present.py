# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_filter_is_present"
# subject = "builtins.filter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.filter: api_filter_is_present (surface)."""
import builtins

assert hasattr(builtins, "filter")
print("api_filter_is_present OK")
