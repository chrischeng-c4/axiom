# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "surface"
# case = "api_dump_is_present"
# subject = "json.dump"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""json.dump: api_dump_is_present (surface)."""
import json

assert hasattr(json, "dump")
print("api_dump_is_present OK")
