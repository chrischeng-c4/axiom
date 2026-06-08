# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_iter_fields_is_present"
# subject = "ast.iter_fields"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.iter_fields: api_iter_fields_is_present (surface)."""
import ast

assert hasattr(ast, "iter_fields")
print("api_iter_fields_is_present OK")
