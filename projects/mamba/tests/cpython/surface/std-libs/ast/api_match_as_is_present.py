# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_match_as_is_present"
# subject = "ast.MatchAs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.MatchAs: api_match_as_is_present (surface)."""
import ast

assert hasattr(ast, "MatchAs")
print("api_match_as_is_present OK")
