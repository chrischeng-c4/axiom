# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_try_is_present"
# subject = "ast.Try"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Try: api_try_is_present (surface)."""
import ast

assert hasattr(ast, "Try")
print("api_try_is_present OK")
