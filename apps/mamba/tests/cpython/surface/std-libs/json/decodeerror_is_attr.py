# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "surface"
# case = "decodeerror_is_attr"
# subject = "json"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""json: decodeerror_is_attr (surface)."""
import json

assert hasattr(json, "JSONDecodeError")
print("decodeerror_is_attr OK")
