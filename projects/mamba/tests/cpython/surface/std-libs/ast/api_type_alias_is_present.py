# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_type_alias_is_present"
# subject = "ast.TypeAlias"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.TypeAlias: api_type_alias_is_present (surface)."""
import ast

assert hasattr(ast, "TypeAlias")
print("api_type_alias_is_present OK")
