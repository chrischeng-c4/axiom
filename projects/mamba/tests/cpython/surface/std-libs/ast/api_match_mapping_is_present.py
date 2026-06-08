# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_match_mapping_is_present"
# subject = "ast.MatchMapping"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.MatchMapping: api_match_mapping_is_present (surface)."""
import ast

assert hasattr(ast, "MatchMapping")
print("api_match_mapping_is_present OK")
