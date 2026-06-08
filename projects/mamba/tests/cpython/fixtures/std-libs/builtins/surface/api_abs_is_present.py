# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_abs_is_present"
# subject = "builtins.abs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.abs: api_abs_is_present (surface)."""
import builtins

assert hasattr(builtins, "abs")
print("api_abs_is_present OK")
