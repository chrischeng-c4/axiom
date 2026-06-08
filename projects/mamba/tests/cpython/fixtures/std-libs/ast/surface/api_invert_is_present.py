# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_invert_is_present"
# subject = "ast.Invert"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Invert: api_invert_is_present (surface)."""
import ast

assert hasattr(ast, "Invert")
print("api_invert_is_present OK")
