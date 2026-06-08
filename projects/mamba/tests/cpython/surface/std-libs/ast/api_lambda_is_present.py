# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_lambda_is_present"
# subject = "ast.Lambda"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Lambda: api_lambda_is_present (surface)."""
import ast

assert hasattr(ast, "Lambda")
print("api_lambda_is_present OK")
