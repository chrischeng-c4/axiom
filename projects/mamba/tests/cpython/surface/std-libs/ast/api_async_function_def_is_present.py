# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_async_function_def_is_present"
# subject = "ast.AsyncFunctionDef"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.AsyncFunctionDef: api_async_function_def_is_present (surface)."""
import ast

assert hasattr(ast, "AsyncFunctionDef")
print("api_async_function_def_is_present OK")
