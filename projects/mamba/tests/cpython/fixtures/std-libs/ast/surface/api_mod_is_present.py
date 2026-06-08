# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_mod_is_present"
# subject = "ast.Mod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Mod: api_mod_is_present (surface)."""
import ast

assert hasattr(ast, "Mod")
print("api_mod_is_present OK")
