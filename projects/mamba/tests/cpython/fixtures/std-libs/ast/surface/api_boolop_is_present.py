# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_boolop_is_present"
# subject = "ast.boolop"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.boolop: api_boolop_is_present (surface)."""
import ast

assert hasattr(ast, "boolop")
print("api_boolop_is_present OK")
