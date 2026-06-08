# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "surface"
# case = "api_json_encoder_is_present"
# subject = "json.JSONEncoder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""json.JSONEncoder: api_json_encoder_is_present (surface)."""
import json

assert hasattr(json, "JSONEncoder")
print("api_json_encoder_is_present OK")
