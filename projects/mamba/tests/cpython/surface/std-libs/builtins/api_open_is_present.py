# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_open_is_present"
# subject = "builtins.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.open: api_open_is_present (surface)."""
import builtins

assert hasattr(builtins, "open")
print("api_open_is_present OK")
