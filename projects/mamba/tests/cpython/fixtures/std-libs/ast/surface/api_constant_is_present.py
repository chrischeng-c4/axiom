# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_constant_is_present"
# subject = "ast.Constant"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Constant: api_constant_is_present (surface)."""
import ast

assert hasattr(ast, "Constant")
print("api_constant_is_present OK")
