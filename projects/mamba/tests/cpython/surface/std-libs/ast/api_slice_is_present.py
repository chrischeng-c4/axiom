# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_slice_is_present"
# subject = "ast.Slice"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Slice: api_slice_is_present (surface)."""
import ast

assert hasattr(ast, "Slice")
print("api_slice_is_present OK")
