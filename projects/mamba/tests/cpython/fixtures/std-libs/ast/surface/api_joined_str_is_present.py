# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_joined_str_is_present"
# subject = "ast.JoinedStr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.JoinedStr: api_joined_str_is_present (surface)."""
import ast

assert hasattr(ast, "JoinedStr")
print("api_joined_str_is_present OK")
