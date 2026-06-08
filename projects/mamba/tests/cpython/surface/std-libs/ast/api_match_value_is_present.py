# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_match_value_is_present"
# subject = "ast.MatchValue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.MatchValue: api_match_value_is_present (surface)."""
import ast

assert hasattr(ast, "MatchValue")
print("api_match_value_is_present OK")
