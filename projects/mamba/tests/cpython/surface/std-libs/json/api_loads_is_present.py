# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "surface"
# case = "api_loads_is_present"
# subject = "json.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""json.loads: api_loads_is_present (surface)."""
import json

assert hasattr(json, "loads")
print("api_loads_is_present OK")
