# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_runtime_warning_is_present"
# subject = "builtins.RuntimeWarning"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.RuntimeWarning: api_runtime_warning_is_present (surface)."""
import builtins

assert hasattr(builtins, "RuntimeWarning")
print("api_runtime_warning_is_present OK")
