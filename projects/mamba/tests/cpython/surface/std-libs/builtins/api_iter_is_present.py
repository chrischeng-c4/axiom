# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_iter_is_present"
# subject = "builtins.iter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.iter: api_iter_is_present (surface)."""
import builtins

assert hasattr(builtins, "iter")
print("api_iter_is_present OK")
