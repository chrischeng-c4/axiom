# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_pattern_is_present"
# subject = "ast.pattern"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.pattern: api_pattern_is_present (surface)."""
import ast

assert hasattr(ast, "pattern")
print("api_pattern_is_present OK")
