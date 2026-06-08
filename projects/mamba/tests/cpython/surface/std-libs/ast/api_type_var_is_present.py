# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_type_var_is_present"
# subject = "ast.TypeVar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.TypeVar: api_type_var_is_present (surface)."""
import ast

assert hasattr(ast, "TypeVar")
print("api_type_var_is_present OK")
