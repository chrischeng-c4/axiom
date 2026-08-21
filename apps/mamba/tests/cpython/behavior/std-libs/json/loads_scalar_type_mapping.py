# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "loads_scalar_type_mapping"
# subject = "json.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_pass1.py"
# status = "filled"
# ///
"""json.loads: JSON tokens map back to Python types: true is True, false is False, null is None, integer->int, decimal->float"""
import json

assert json.loads("true") is True, "true -> True"
assert json.loads("false") is False, "false -> False"
assert json.loads("null") is None, "null -> None"
assert isinstance(json.loads("1"), int), "integer -> int"
assert json.loads("42") == 42, json.loads("42")
assert isinstance(json.loads("1.5"), float), "decimal -> float"
assert json.loads('"hello"') == "hello", json.loads('"hello"')

print("loads_scalar_type_mapping OK")
