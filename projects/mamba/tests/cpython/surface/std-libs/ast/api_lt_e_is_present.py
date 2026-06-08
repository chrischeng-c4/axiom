# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_lt_e_is_present"
# subject = "ast.LtE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.LtE: api_lt_e_is_present (surface)."""
import ast

assert hasattr(ast, "LtE")
print("api_lt_e_is_present OK")
