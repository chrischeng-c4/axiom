# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_withitem_is_present"
# subject = "ast.withitem"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.withitem: api_withitem_is_present (surface)."""
import ast

assert hasattr(ast, "withitem")
print("api_withitem_is_present OK")
