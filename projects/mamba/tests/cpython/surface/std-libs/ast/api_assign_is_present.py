# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_assign_is_present"
# subject = "ast.Assign"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Assign: api_assign_is_present (surface)."""
import ast

assert hasattr(ast, "Assign")
print("api_assign_is_present OK")
