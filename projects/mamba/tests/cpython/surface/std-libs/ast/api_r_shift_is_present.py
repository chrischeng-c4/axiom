# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_r_shift_is_present"
# subject = "ast.RShift"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.RShift: api_r_shift_is_present (surface)."""
import ast

assert hasattr(ast, "RShift")
print("api_r_shift_is_present OK")
