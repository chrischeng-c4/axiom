# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_keyword_is_present"
# subject = "ast.keyword"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.keyword: api_keyword_is_present (surface)."""
import ast

assert hasattr(ast, "keyword")
print("api_keyword_is_present OK")
