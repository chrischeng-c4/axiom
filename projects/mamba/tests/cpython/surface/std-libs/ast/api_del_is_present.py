# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_del_is_present"
# subject = "ast.Del"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Del: api_del_is_present (surface)."""
import ast

assert hasattr(ast, "Del")
print("api_del_is_present OK")
