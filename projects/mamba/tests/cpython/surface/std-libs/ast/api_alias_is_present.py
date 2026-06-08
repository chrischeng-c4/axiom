# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_alias_is_present"
# subject = "ast.alias"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.alias: api_alias_is_present (surface)."""
import ast

assert hasattr(ast, "alias")
print("api_alias_is_present OK")
