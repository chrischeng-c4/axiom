# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_gt_e_is_present"
# subject = "ast.GtE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.GtE: api_gt_e_is_present (surface)."""
import ast

assert hasattr(ast, "GtE")
print("api_gt_e_is_present OK")
