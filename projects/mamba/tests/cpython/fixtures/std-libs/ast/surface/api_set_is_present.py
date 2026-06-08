# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_set_is_present"
# subject = "ast.Set"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Set: api_set_is_present (surface)."""
import ast

assert hasattr(ast, "Set")
print("api_set_is_present OK")
