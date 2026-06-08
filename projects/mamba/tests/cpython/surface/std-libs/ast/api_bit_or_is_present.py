# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_bit_or_is_present"
# subject = "ast.BitOr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.BitOr: api_bit_or_is_present (surface)."""
import ast

assert hasattr(ast, "BitOr")
print("api_bit_or_is_present OK")
