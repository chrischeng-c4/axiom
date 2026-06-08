# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_indentation_error_is_present"
# subject = "builtins.IndentationError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.IndentationError: api_indentation_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "IndentationError")
print("api_indentation_error_is_present OK")
