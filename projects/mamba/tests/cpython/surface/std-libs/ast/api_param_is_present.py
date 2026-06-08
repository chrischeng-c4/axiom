# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_param_is_present"
# subject = "ast.Param"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Param: api_param_is_present (surface)."""
import ast

assert hasattr(ast, "Param")
print("api_param_is_present OK")
