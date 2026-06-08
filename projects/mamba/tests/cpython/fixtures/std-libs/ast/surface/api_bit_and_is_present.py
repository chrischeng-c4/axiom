# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_bit_and_is_present"
# subject = "ast.BitAnd"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.BitAnd: api_bit_and_is_present (surface)."""
import ast

assert hasattr(ast, "BitAnd")
print("api_bit_and_is_present OK")
