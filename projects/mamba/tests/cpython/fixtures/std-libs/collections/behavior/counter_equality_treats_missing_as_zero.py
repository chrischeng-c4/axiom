# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_equality_treats_missing_as_zero"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: Counter equality treats a missing key as count 0, so Counter(a=3, b=2, c=0) == Counter('ababa')"""
from collections import Counter

assert Counter(a=3, b=2, c=0) == Counter("ababa"), "equality treats missing/zero alike"
assert Counter(a=3, b=2) != Counter("babab"), "different tallies are unequal"

print("counter_equality_treats_missing_as_zero OK")
