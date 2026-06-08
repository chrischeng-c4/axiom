# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_yield_from_is_present"
# subject = "ast.YieldFrom"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.YieldFrom: api_yield_from_is_present (surface)."""
import ast

assert hasattr(ast, "YieldFrom")
print("api_yield_from_is_present OK")
