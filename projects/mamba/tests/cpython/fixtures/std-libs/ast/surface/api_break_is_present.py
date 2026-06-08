# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_break_is_present"
# subject = "ast.Break"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Break: api_break_is_present (surface)."""
import ast

assert hasattr(ast, "Break")
print("api_break_is_present OK")
