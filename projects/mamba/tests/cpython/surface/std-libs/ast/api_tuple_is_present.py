# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_tuple_is_present"
# subject = "ast.Tuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Tuple: api_tuple_is_present (surface)."""
import ast

assert hasattr(ast, "Tuple")
print("api_tuple_is_present OK")
