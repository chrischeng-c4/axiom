# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_param_spec_is_present"
# subject = "ast.ParamSpec"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.ParamSpec: api_param_spec_is_present (surface)."""
import ast

assert hasattr(ast, "ParamSpec")
print("api_param_spec_is_present OK")
