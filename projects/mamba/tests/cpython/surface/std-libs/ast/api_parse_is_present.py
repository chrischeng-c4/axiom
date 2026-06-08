# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_parse_is_present"
# subject = "ast.parse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.parse: api_parse_is_present (surface)."""
import ast

assert hasattr(ast, "parse")
print("api_parse_is_present OK")
