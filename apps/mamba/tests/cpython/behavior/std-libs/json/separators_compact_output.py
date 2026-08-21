# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "separators_compact_output"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_separators.py"
# status = "filled"
# ///
"""json.dumps: separators=(',',':') drops all inter-token whitespace producing the most compact serialization"""
import json

assert json.dumps([1, 2], separators=(",", ":")) == "[1,2]", json.dumps([1, 2], separators=(",", ":"))
compact = json.dumps({"a": 1, "b": 2}, separators=(",", ":"), sort_keys=True)
assert compact == '{"a":1,"b":2}', f"compact = {compact!r}"

print("separators_compact_output OK")
