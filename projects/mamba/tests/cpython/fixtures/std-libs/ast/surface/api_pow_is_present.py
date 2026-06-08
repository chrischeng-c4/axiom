# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_pow_is_present"
# subject = "ast.Pow"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Pow: api_pow_is_present (surface)."""
import ast

assert hasattr(ast, "Pow")
print("api_pow_is_present OK")
