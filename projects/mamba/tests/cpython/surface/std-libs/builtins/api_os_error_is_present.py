# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_os_error_is_present"
# subject = "builtins.OSError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.OSError: api_os_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "OSError")
print("api_os_error_is_present OK")
