# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "sort_keys_orders_object"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_separators.py"
# status = "filled"
# ///
"""json.dumps: sort_keys=True emits object keys in deterministic ascending order regardless of insertion order"""
import json

assert json.dumps({"b": 2, "a": 1}, sort_keys=True) == '{"a": 1, "b": 2}'
assert json.dumps({"b": 2, "a": 1, "c": 3}, sort_keys=True) == '{"a": 1, "b": 2, "c": 3}'
assert json.dumps({"z": 1, "a": 2, "m": 3}, sort_keys=True) == '{"a": 2, "m": 3, "z": 1}'

print("sort_keys_orders_object OK")
