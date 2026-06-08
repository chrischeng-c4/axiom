# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_nonlocal_is_present"
# subject = "ast.Nonlocal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Nonlocal: api_nonlocal_is_present (surface)."""
import ast

assert hasattr(ast, "Nonlocal")
print("api_nonlocal_is_present OK")
