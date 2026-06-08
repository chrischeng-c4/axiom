# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_sorted_is_present"
# subject = "builtins.sorted"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.sorted: api_sorted_is_present (surface)."""
import builtins

assert hasattr(builtins, "sorted")
print("api_sorted_is_present OK")
