# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_continue_is_present"
# subject = "ast.Continue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Continue: api_continue_is_present (surface)."""
import ast

assert hasattr(ast, "Continue")
print("api_continue_is_present OK")
