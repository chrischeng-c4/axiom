# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_is_not_is_present"
# subject = "ast.IsNot"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.IsNot: api_is_not_is_present (surface)."""
import ast

assert hasattr(ast, "IsNot")
print("api_is_not_is_present OK")
