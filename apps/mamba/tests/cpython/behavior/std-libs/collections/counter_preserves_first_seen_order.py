# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_preserves_first_seen_order"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: Counter preserves the first-occurrence insertion order of its keys: Counter('abracadabra').items() == [('a',5),('b',2),('r',2),('c',1),('d',1)]"""
from collections import Counter

assert list(Counter("abracadabra").items()) == [
    ("a", 5), ("b", 2), ("r", 2), ("c", 1), ("d", 1),
], "keys keep first-occurrence order"

print("counter_preserves_first_seen_order OK")
