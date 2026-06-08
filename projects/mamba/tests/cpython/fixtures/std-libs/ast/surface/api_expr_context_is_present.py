# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_expr_context_is_present"
# subject = "ast.expr_context"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.expr_context: api_expr_context_is_present (surface)."""
import ast

assert hasattr(ast, "expr_context")
print("api_expr_context_is_present OK")
