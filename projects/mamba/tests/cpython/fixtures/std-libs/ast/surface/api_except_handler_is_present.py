# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_except_handler_is_present"
# subject = "ast.ExceptHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.ExceptHandler: api_except_handler_is_present (surface)."""
import ast

assert hasattr(ast, "ExceptHandler")
print("api_except_handler_is_present OK")
