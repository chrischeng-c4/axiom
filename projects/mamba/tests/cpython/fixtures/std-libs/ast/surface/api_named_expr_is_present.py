# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_named_expr_is_present"
# subject = "ast.NamedExpr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.NamedExpr: api_named_expr_is_present (surface)."""
import ast

assert hasattr(ast, "NamedExpr")
print("api_named_expr_is_present OK")
