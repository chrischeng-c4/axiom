# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_dict_is_present"
# subject = "ast.Dict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Dict: api_dict_is_present (surface)."""
import ast

assert hasattr(ast, "Dict")
print("api_dict_is_present OK")
