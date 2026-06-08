# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_unary_op_is_present"
# subject = "ast.UnaryOp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.UnaryOp: api_unary_op_is_present (surface)."""
import ast

assert hasattr(ast, "UnaryOp")
print("api_unary_op_is_present OK")
