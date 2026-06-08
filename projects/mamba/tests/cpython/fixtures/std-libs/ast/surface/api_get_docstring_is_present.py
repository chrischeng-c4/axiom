# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_get_docstring_is_present"
# subject = "ast.get_docstring"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.get_docstring: api_get_docstring_is_present (surface)."""
import ast

assert hasattr(ast, "get_docstring")
print("api_get_docstring_is_present OK")
