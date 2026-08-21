# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "dumps_scalar_type_mapping"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_pass1.py"
# status = "filled"
# ///
"""json.dumps: Python scalars map to JSON tokens: True->true, False->false, None->null, int->number, float->number, str->quoted"""
import json

assert json.dumps(True) == "true", json.dumps(True)
assert json.dumps(False) == "false", json.dumps(False)
assert json.dumps(None) == "null", json.dumps(None)
assert json.dumps(1) == "1", json.dumps(1)
assert json.dumps(42) == "42", json.dumps(42)
assert json.dumps(1.5) == "1.5", json.dumps(1.5)
assert json.dumps(3.14) == "3.14", json.dumps(3.14)
assert json.dumps("hello") == '"hello"', json.dumps("hello")

print("dumps_scalar_type_mapping OK")
