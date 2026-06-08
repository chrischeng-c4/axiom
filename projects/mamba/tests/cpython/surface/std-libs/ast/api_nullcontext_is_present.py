# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_nullcontext_is_present"
# subject = "ast.nullcontext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.nullcontext: api_nullcontext_is_present (surface)."""
import ast

assert hasattr(ast, "nullcontext")
print("api_nullcontext_is_present OK")
