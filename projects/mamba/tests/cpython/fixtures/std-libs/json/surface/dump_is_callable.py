# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "surface"
# case = "dump_is_callable"
# subject = "json.dump"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""json.dump: dump_is_callable (surface)."""
import json

assert callable(json.dump)
print("dump_is_callable OK")
