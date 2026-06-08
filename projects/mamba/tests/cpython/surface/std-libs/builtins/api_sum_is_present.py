# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_sum_is_present"
# subject = "builtins.sum"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.sum: api_sum_is_present (surface)."""
import builtins

assert hasattr(builtins, "sum")
print("api_sum_is_present OK")
