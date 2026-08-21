# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_counts_from_iterable"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: Counter('abracadabra') tallies element frequencies; a present key returns its count and a missing key returns 0 without being inserted"""
from collections import Counter

c = Counter("abracadabra")
assert c["a"] == 5, f"a = {c['a']!r}"
assert c["b"] == 2, f"b = {c['b']!r}"
assert c["r"] == 2, f"r = {c['r']!r}"
assert c["z"] == 0, "missing key reads as 0"
assert "z" not in c, "reading a missing key does not insert it"

print("counter_counts_from_iterable OK")
