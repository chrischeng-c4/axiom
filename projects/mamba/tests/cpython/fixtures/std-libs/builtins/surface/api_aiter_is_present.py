# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_aiter_is_present"
# subject = "builtins.aiter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.aiter: api_aiter_is_present (surface)."""
import builtins

assert hasattr(builtins, "aiter")
print("api_aiter_is_present OK")
