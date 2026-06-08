# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_global_is_present"
# subject = "ast.Global"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Global: api_global_is_present (surface)."""
import ast

assert hasattr(ast, "Global")
print("api_global_is_present OK")
