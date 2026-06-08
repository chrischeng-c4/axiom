# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_type_ignore_is_present_2"
# subject = "ast.type_ignore"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.type_ignore: api_type_ignore_is_present_2 (surface)."""
import ast

assert hasattr(ast, "type_ignore")
print("api_type_ignore_is_present_2 OK")
