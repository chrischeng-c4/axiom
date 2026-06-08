# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_io_error_is_present"
# subject = "builtins.IOError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.IOError: api_io_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "IOError")
print("api_io_error_is_present OK")
