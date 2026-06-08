# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_await_is_present"
# subject = "ast.Await"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Await: api_await_is_present (surface)."""
import ast

assert hasattr(ast, "Await")
print("api_await_is_present OK")
