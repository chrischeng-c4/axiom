# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "surface"
# case = "load_is_callable"
# subject = "json.load"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""json.load: load_is_callable (surface)."""
import json

assert callable(json.load)
print("load_is_callable OK")
