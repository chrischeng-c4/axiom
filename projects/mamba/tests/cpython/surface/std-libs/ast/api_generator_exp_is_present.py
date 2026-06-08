# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_generator_exp_is_present"
# subject = "ast.GeneratorExp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.GeneratorExp: api_generator_exp_is_present (surface)."""
import ast

assert hasattr(ast, "GeneratorExp")
print("api_generator_exp_is_present OK")
