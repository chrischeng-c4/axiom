# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_type_var_tuple_is_present"
# subject = "ast.TypeVarTuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.TypeVarTuple: api_type_var_tuple_is_present (surface)."""
import ast

assert hasattr(ast, "TypeVarTuple")
print("api_type_var_tuple_is_present OK")
