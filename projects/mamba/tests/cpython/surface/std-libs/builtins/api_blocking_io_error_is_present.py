# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_blocking_io_error_is_present"
# subject = "builtins.BlockingIOError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.BlockingIOError: api_blocking_io_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "BlockingIOError")
print("api_blocking_io_error_is_present OK")
