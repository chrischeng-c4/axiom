# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_type_param_is_present"
# subject = "ast.type_param"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.type_param: api_type_param_is_present (surface)."""
import ast

assert hasattr(ast, "type_param")
print("api_type_param_is_present OK")
