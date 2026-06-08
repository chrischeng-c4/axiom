# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_not_in_is_present"
# subject = "ast.NotIn"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.NotIn: api_not_in_is_present (surface)."""
import ast

assert hasattr(ast, "NotIn")
print("api_not_in_is_present OK")
