# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_module_is_present"
# subject = "ast.Module"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Module: api_module_is_present (surface)."""
import ast

assert hasattr(ast, "Module")
print("api_module_is_present OK")
