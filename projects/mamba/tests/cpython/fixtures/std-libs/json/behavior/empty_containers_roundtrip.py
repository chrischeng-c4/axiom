# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "empty_containers_roundtrip"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""json.dumps: empty list and empty dict serialize to [] and {} and load back to equal empty containers"""
import json

assert json.dumps([]) == "[]", json.dumps([])
assert json.dumps({}) == "{}", json.dumps({})
assert json.loads("[]") == [], json.loads("[]")
assert json.loads("{}") == {}, json.loads("{}")

print("empty_containers_roundtrip OK")
