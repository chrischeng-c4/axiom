# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_async_with_is_present"
# subject = "ast.AsyncWith"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.AsyncWith: api_async_with_is_present (surface)."""
import ast

assert hasattr(ast, "AsyncWith")
print("api_async_with_is_present OK")
