# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_literal_eval_is_present"
# subject = "ast.literal_eval"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.literal_eval: api_literal_eval_is_present (surface)."""
import ast

assert hasattr(ast, "literal_eval")
print("api_literal_eval_is_present OK")
