# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "nested_structure_roundtrip"
# subject = "json.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_pass1.py"
# status = "filled"
# ///
"""json.loads: a deeply nested dict/list/scalar tree survives dumps->loads byte-for-value round-trip unchanged"""
import json

nested = {"a": {"b": {"c": [1, 2, {"d": 3}]}}}
rt = json.loads(json.dumps(nested))
assert rt == nested, f"nested round-trip = {rt!r}"

deeper = {"outer": {"inner": 42}, "list": [1, 2, {"y": "nested"}]}
back = json.loads(json.dumps(deeper, sort_keys=True))
assert back["outer"]["inner"] == 42, back
assert back["list"][2]["y"] == "nested", back

print("nested_structure_roundtrip OK")
