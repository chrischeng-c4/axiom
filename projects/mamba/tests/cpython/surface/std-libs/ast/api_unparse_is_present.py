# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_unparse_is_present"
# subject = "ast.unparse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.unparse: api_unparse_is_present (surface)."""
import ast

assert hasattr(ast, "unparse")
print("api_unparse_is_present OK")
