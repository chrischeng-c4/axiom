# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_if_exp_is_present"
# subject = "ast.IfExp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.IfExp: api_if_exp_is_present (surface)."""
import ast

assert hasattr(ast, "IfExp")
print("api_if_exp_is_present OK")
