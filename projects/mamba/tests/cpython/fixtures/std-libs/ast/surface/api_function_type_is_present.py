# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_function_type_is_present"
# subject = "ast.FunctionType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.FunctionType: api_function_type_is_present (surface)."""
import ast

assert hasattr(ast, "FunctionType")
print("api_function_type_is_present OK")
