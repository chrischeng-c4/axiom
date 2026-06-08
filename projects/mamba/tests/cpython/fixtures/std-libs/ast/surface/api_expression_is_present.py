# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_expression_is_present"
# subject = "ast.Expression"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Expression: api_expression_is_present (surface)."""
import ast

assert hasattr(ast, "Expression")
print("api_expression_is_present OK")
