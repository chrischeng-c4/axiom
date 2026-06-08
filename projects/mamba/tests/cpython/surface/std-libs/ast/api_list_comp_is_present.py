# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_list_comp_is_present"
# subject = "ast.ListComp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.ListComp: api_list_comp_is_present (surface)."""
import ast

assert hasattr(ast, "ListComp")
print("api_list_comp_is_present OK")
