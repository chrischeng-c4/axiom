# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_any_is_present"
# subject = "builtins.any"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.any: api_any_is_present (surface)."""
import builtins

assert hasattr(builtins, "any")
print("api_any_is_present OK")
