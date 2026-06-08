# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_match_case_is_present"
# subject = "ast.match_case"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.match_case: api_match_case_is_present (surface)."""
import ast

assert hasattr(ast, "match_case")
print("api_match_case_is_present OK")
