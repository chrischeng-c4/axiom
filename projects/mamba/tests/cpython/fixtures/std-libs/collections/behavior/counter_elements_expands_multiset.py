# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_elements_expands_multiset"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: Counter.elements() yields each element repeated by its count, and sorted(elements()) round-trips a source string as a multiset"""
from collections import Counter

assert sorted(Counter(a=2, b=3).elements()) == ["a", "a", "b", "b", "b"], "elements repeat by count"
s = "she sells sea shells"
assert sorted(Counter(s).elements()) == sorted(s), "elements round-trips the source as a multiset"
assert set(Counter(s)) == set(s), "keys are the distinct elements"

print("counter_elements_expands_multiset OK")
