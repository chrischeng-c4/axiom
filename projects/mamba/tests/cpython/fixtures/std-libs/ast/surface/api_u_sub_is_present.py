# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_u_sub_is_present"
# subject = "ast.USub"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.USub: api_u_sub_is_present (surface)."""
import ast

assert hasattr(ast, "USub")
print("api_u_sub_is_present OK")
