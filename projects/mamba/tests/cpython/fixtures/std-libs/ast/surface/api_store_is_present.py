# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_store_is_present"
# subject = "ast.Store"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Store: api_store_is_present (surface)."""
import ast

assert hasattr(ast, "Store")
print("api_store_is_present OK")
