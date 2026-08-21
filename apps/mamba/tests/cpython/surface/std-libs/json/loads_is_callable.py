# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "surface"
# case = "loads_is_callable"
# subject = "json.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""json.loads: loads_is_callable (surface)."""
import json

assert callable(json.loads)
print("loads_is_callable OK")
