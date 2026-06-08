# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_arg_is_present"
# subject = "ast.arg"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.arg: api_arg_is_present (surface)."""
import ast

assert hasattr(ast, "arg")
print("api_arg_is_present OK")
