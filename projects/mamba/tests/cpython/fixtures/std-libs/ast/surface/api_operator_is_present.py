# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_operator_is_present"
# subject = "ast.operator"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.operator: api_operator_is_present (surface)."""
import ast

assert hasattr(ast, "operator")
print("api_operator_is_present OK")
