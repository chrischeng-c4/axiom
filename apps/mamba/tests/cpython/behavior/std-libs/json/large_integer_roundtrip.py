# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "large_integer_roundtrip"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""json.dumps: a large integer (10**15) survives the dumps->loads round-trip with exact value and int type preserved"""
import json

big = 10 ** 15
rt = json.loads(json.dumps(big))
assert rt == big, f"big int round-trip = {rt!r}"
assert isinstance(rt, int), rt

print("large_integer_roundtrip OK")
