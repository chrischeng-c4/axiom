# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_subscript_is_present"
# subject = "ast.Subscript"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Subscript: api_subscript_is_present (surface)."""
import ast

assert hasattr(ast, "Subscript")
print("api_subscript_is_present OK")
