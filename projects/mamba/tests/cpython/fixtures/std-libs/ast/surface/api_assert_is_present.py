# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_assert_is_present"
# subject = "ast.Assert"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Assert: api_assert_is_present (surface)."""
import ast

assert hasattr(ast, "Assert")
print("api_assert_is_present OK")
