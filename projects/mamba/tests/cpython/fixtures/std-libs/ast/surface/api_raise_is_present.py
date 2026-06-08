# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_raise_is_present"
# subject = "ast.Raise"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Raise: api_raise_is_present (surface)."""
import ast

assert hasattr(ast, "Raise")
print("api_raise_is_present OK")
