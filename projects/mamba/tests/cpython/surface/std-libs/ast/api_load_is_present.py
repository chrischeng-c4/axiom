# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_load_is_present"
# subject = "ast.Load"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Load: api_load_is_present (surface)."""
import ast

assert hasattr(ast, "Load")
print("api_load_is_present OK")
