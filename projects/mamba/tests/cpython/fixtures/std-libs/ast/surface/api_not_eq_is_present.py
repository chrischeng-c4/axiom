# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_not_eq_is_present"
# subject = "ast.NotEq"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.NotEq: api_not_eq_is_present (surface)."""
import ast

assert hasattr(ast, "NotEq")
print("api_not_eq_is_present OK")
