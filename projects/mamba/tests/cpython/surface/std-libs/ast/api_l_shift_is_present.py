# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_l_shift_is_present"
# subject = "ast.LShift"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.LShift: api_l_shift_is_present (surface)."""
import ast

assert hasattr(ast, "LShift")
print("api_l_shift_is_present OK")
