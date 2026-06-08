# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_copy_location_is_present"
# subject = "ast.copy_location"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.copy_location: api_copy_location_is_present (surface)."""
import ast

assert hasattr(ast, "copy_location")
print("api_copy_location_is_present OK")
