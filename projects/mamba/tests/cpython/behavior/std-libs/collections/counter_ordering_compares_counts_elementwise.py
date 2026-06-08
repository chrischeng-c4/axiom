# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_ordering_compares_counts_elementwise"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: multiset ordering <, <=, >, >= compare counts elementwise (missing == 0) across a representative table of Counter pairs"""
from collections import Counter

assert Counter(a=3, b=1, c=0) < Counter("ababa"), "lt"
assert not (Counter(a=3, b=2, c=0) < Counter("ababa")), "not lt when equal"
assert Counter(a=3, b=2, c=0) <= Counter("ababa"), "le"
assert Counter(a=3, b=2, c=0) > Counter("aab"), "gt"
assert not (Counter(a=2, b=1, c=0) > Counter("aab")), "not gt when equal"
assert Counter(a=2, b=1, c=0) >= Counter("aab"), "ge"
assert not (Counter(a=3, b=2, c=0) >= Counter("aabd")), "not ge when a key is missing"

print("counter_ordering_compares_counts_elementwise OK")
