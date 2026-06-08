# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "surface"
# case = "decoder_is_callable"
# subject = "json.JSONDecoder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""json.JSONDecoder: decoder_is_callable (surface)."""
import json

assert callable(json.JSONDecoder)
print("decoder_is_callable OK")
