# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_sys_is_present"
# subject = "ast.sys"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.sys: api_sys_is_present (surface)."""
import ast

assert hasattr(ast, "sys")
print("api_sys_is_present OK")
