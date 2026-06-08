# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_broken_pipe_error_is_present"
# subject = "builtins.BrokenPipeError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.BrokenPipeError: api_broken_pipe_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "BrokenPipeError")
print("api_broken_pipe_error_is_present OK")
