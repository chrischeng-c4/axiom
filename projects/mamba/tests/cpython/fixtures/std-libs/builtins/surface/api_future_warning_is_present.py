# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_future_warning_is_present"
# subject = "builtins.FutureWarning"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.FutureWarning: api_future_warning_is_present (surface)."""
import builtins

assert hasattr(builtins, "FutureWarning")
print("api_future_warning_is_present OK")
