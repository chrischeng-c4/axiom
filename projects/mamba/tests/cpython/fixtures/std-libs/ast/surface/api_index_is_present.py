# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_index_is_present"
# subject = "ast.Index"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Index: api_index_is_present (surface)."""
import ast

assert hasattr(ast, "Index")
print("api_index_is_present OK")
