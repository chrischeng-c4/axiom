# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_delete_is_present"
# subject = "ast.Delete"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Delete: api_delete_is_present (surface)."""
import ast

assert hasattr(ast, "Delete")
print("api_delete_is_present OK")
