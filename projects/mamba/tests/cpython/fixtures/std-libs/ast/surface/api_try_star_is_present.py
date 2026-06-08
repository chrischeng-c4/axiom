# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_try_star_is_present"
# subject = "ast.TryStar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.TryStar: api_try_star_is_present (surface)."""
import ast

assert hasattr(ast, "TryStar")
print("api_try_star_is_present OK")
