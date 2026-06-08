# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_multiset_ops_match_set_ops"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: on counts of 0/1 the multiset operators +, -, |, & agree with the corresponding set operations on the present elements"""
from collections import Counter

def to_set(counter):
    return set(counter.elements())

p = Counter(a=1, b=1, c=0)
q = Counter(b=1, c=1, d=1)
assert set((p + q).elements()) == to_set(p) | to_set(q), "add ~ union"
assert set((p - q).elements()) == to_set(p) - to_set(q), "sub ~ difference"
assert set((p | q).elements()) == to_set(p) | to_set(q), "or ~ union"
assert set((p & q).elements()) == to_set(p) & to_set(q), "and ~ intersection"

print("counter_multiset_ops_match_set_ops OK")
