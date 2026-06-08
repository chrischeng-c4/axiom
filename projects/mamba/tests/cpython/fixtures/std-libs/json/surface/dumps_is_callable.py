# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "surface"
# case = "dumps_is_callable"
# subject = "json.dumps"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""json.dumps: dumps_is_callable (surface)."""
import json

assert callable(json.dumps)
print("dumps_is_callable OK")
