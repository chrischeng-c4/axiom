# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_suite_is_present"
# subject = "ast.Suite"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Suite: api_suite_is_present (surface)."""
import ast

assert hasattr(ast, "Suite")
print("api_suite_is_present OK")
