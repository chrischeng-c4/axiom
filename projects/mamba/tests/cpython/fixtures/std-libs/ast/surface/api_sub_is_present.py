# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_sub_is_present"
# subject = "ast.Sub"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Sub: api_sub_is_present (surface)."""
import ast

assert hasattr(ast, "Sub")
print("api_sub_is_present OK")
