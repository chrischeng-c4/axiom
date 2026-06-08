# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_match_star_is_present"
# subject = "ast.MatchStar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.MatchStar: api_match_star_is_present (surface)."""
import ast

assert hasattr(ast, "MatchStar")
print("api_match_star_is_present OK")
