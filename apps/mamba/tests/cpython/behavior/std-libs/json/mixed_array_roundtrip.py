# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "mixed_array_roundtrip"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_pass1.py"
# status = "filled"
# ///
"""json.dumps: a heterogeneous array of int/str/float/bool/None/dict round-trips through dumps->loads unchanged"""
import json

mixed = [1, "two", 3.0, True, None, {"key": "val"}]
rt = json.loads(json.dumps(mixed))
assert rt == mixed, f"mixed array = {rt!r}"
assert rt[4] is None, rt

print("mixed_array_roundtrip OK")
