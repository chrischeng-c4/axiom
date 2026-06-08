# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_formatted_value_is_present"
# subject = "ast.FormattedValue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.FormattedValue: api_formatted_value_is_present (surface)."""
import ast

assert hasattr(ast, "FormattedValue")
print("api_formatted_value_is_present OK")
