# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_lambda_type_is_present"
# subject = "types.LambdaType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.LambdaType: api_lambda_type_is_present (surface)."""
import types

assert hasattr(types, "LambdaType")
print("api_lambda_type_is_present OK")
