# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_function_def_is_present"
# subject = "ast.FunctionDef"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.FunctionDef: api_function_def_is_present (surface)."""
import ast

assert hasattr(ast, "FunctionDef")
print("api_function_def_is_present OK")
