# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_auto_is_present"
# subject = "ast.auto"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.auto: api_auto_is_present (surface)."""
import ast

assert hasattr(ast, "auto")
print("api_auto_is_present OK")
