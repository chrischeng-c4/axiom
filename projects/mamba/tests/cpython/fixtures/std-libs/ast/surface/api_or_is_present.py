# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_or_is_present"
# subject = "ast.Or"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Or: api_or_is_present (surface)."""
import ast

assert hasattr(ast, "Or")
print("api_or_is_present OK")
