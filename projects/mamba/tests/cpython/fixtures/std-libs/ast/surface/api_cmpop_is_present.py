# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_cmpop_is_present"
# subject = "ast.cmpop"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.cmpop: api_cmpop_is_present (surface)."""
import ast

assert hasattr(ast, "cmpop")
print("api_cmpop_is_present OK")
