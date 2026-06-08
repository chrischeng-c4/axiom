# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_match_sequence_is_present"
# subject = "ast.MatchSequence"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.MatchSequence: api_match_sequence_is_present (surface)."""
import ast

assert hasattr(ast, "MatchSequence")
print("api_match_sequence_is_present OK")
