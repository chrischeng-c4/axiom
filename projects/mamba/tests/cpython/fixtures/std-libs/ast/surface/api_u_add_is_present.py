# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_u_add_is_present"
# subject = "ast.UAdd"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.UAdd: api_u_add_is_present (surface)."""
import ast

assert hasattr(ast, "UAdd")
print("api_u_add_is_present OK")
