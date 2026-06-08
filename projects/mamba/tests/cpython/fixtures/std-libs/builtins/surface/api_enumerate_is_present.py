# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_enumerate_is_present"
# subject = "builtins.enumerate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.enumerate: api_enumerate_is_present (surface)."""
import builtins

assert hasattr(builtins, "enumerate")
print("api_enumerate_is_present OK")
