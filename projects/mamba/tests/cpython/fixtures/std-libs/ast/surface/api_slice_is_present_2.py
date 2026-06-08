# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_slice_is_present_2"
# subject = "ast.slice"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.slice: api_slice_is_present_2 (surface)."""
import ast

assert hasattr(ast, "slice")
print("api_slice_is_present_2 OK")
