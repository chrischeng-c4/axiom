# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_set_comp_is_present"
# subject = "ast.SetComp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.SetComp: api_set_comp_is_present (surface)."""
import ast

assert hasattr(ast, "SetComp")
print("api_set_comp_is_present OK")
