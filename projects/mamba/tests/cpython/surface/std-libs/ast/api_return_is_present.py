# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_return_is_present"
# subject = "ast.Return"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Return: api_return_is_present (surface)."""
import ast

assert hasattr(ast, "Return")
print("api_return_is_present OK")
