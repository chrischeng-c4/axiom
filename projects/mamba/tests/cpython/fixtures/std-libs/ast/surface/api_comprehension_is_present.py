# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_comprehension_is_present"
# subject = "ast.comprehension"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.comprehension: api_comprehension_is_present (surface)."""
import ast

assert hasattr(ast, "comprehension")
print("api_comprehension_is_present OK")
