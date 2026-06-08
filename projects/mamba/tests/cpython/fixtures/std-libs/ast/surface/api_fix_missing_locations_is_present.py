# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_fix_missing_locations_is_present"
# subject = "ast.fix_missing_locations"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.fix_missing_locations: api_fix_missing_locations_is_present (surface)."""
import ast

assert hasattr(ast, "fix_missing_locations")
print("api_fix_missing_locations_is_present OK")
