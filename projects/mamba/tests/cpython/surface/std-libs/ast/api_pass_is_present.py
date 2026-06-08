# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_pass_is_present"
# subject = "ast.Pass"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Pass: api_pass_is_present (surface)."""
import ast

assert hasattr(ast, "Pass")
print("api_pass_is_present OK")
