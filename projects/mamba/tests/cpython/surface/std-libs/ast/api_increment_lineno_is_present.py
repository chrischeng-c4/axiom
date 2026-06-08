# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_increment_lineno_is_present"
# subject = "ast.increment_lineno"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.increment_lineno: api_increment_lineno_is_present (surface)."""
import ast

assert hasattr(ast, "increment_lineno")
print("api_increment_lineno_is_present OK")
