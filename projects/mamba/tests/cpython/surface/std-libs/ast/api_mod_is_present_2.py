# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_mod_is_present_2"
# subject = "ast.mod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.mod: api_mod_is_present_2 (surface)."""
import ast

assert hasattr(ast, "mod")
print("api_mod_is_present_2 OK")
