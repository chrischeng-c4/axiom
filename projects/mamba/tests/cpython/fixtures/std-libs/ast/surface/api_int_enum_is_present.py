# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_int_enum_is_present"
# subject = "ast.IntEnum"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.IntEnum: api_int_enum_is_present (surface)."""
import ast

assert hasattr(ast, "IntEnum")
print("api_int_enum_is_present OK")
