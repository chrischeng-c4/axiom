# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_call_is_present"
# subject = "ast.Call"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Call: api_call_is_present (surface)."""
import ast

assert hasattr(ast, "Call")
print("api_call_is_present OK")
