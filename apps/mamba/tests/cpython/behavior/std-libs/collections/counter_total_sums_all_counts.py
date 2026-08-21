# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_total_sums_all_counts"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: Counter.total() sums every count including zero and negative ones (Counter(a=10, b=5, c=0).total() == 15)"""
from collections import Counter

assert Counter(a=10, b=5, c=0).total() == 15, "total sums every count incl. zero"
assert Counter(a=-5, b=0, c=5).total() == 0, "total includes negative counts"

print("counter_total_sums_all_counts OK")
