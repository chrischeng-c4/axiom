# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_eq_is_present"
# subject = "ast.Eq"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Eq: api_eq_is_present (surface)."""
import ast

assert hasattr(ast, "Eq")
print("api_eq_is_present OK")
