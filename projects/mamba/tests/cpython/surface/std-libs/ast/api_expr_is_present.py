# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_expr_is_present"
# subject = "ast.Expr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Expr: api_expr_is_present (surface)."""
import ast

assert hasattr(ast, "Expr")
print("api_expr_is_present OK")
