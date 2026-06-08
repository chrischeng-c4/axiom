# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_buffer_error_is_present"
# subject = "builtins.BufferError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.BufferError: api_buffer_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "BufferError")
print("api_buffer_error_is_present OK")
