# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_syntax_warning_is_present"
# subject = "builtins.SyntaxWarning"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.SyntaxWarning: api_syntax_warning_is_present (surface)."""
import builtins

assert hasattr(builtins, "SyntaxWarning")
print("api_syntax_warning_is_present OK")
