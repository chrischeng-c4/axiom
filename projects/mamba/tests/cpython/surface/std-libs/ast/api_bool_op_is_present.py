# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_bool_op_is_present"
# subject = "ast.BoolOp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.BoolOp: api_bool_op_is_present (surface)."""
import ast

assert hasattr(ast, "BoolOp")
print("api_bool_op_is_present OK")
