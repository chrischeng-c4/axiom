# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "surface"
# case = "encoder_is_callable"
# subject = "json.JSONEncoder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""json.JSONEncoder: encoder_is_callable (surface)."""
import json

assert callable(json.JSONEncoder)
print("encoder_is_callable OK")
