# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_breakpoint_is_present"
# subject = "builtins.breakpoint"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.breakpoint: api_breakpoint_is_present (surface)."""
import builtins

assert hasattr(builtins, "breakpoint")
print("api_breakpoint_is_present OK")
