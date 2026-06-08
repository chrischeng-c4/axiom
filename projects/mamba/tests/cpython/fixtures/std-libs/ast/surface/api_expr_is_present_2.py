# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_expr_is_present_2"
# subject = "ast.expr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.expr: api_expr_is_present_2 (surface)."""
import ast

assert hasattr(ast, "expr")
print("api_expr_is_present_2 OK")
