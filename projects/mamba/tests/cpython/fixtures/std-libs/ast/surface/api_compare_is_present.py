# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_compare_is_present"
# subject = "ast.Compare"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Compare: api_compare_is_present (surface)."""
import ast

assert hasattr(ast, "Compare")
print("api_compare_is_present OK")
