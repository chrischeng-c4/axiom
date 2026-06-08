# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_syntax_error_is_present"
# subject = "builtins.SyntaxError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.SyntaxError: api_syntax_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "SyntaxError")
print("api_syntax_error_is_present OK")
