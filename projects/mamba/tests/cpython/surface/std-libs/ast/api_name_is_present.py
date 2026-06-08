# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_name_is_present"
# subject = "ast.Name"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Name: api_name_is_present (surface)."""
import ast

assert hasattr(ast, "Name")
print("api_name_is_present OK")
