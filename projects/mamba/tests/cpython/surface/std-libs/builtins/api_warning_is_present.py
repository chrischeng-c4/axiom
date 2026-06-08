# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_warning_is_present"
# subject = "builtins.Warning"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.Warning: api_warning_is_present (surface)."""
import builtins

assert hasattr(builtins, "Warning")
print("api_warning_is_present OK")
