# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_eof_error_is_present"
# subject = "builtins.EOFError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.EOFError: api_eof_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "EOFError")
print("api_eof_error_is_present OK")
