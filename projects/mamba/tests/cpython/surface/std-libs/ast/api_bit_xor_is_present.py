# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_bit_xor_is_present"
# subject = "ast.BitXor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.BitXor: api_bit_xor_is_present (surface)."""
import ast

assert hasattr(ast, "BitXor")
print("api_bit_xor_is_present OK")
