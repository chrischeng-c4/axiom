# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_import_from_is_present"
# subject = "ast.ImportFrom"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.ImportFrom: api_import_from_is_present (surface)."""
import ast

assert hasattr(ast, "ImportFrom")
print("api_import_from_is_present OK")
