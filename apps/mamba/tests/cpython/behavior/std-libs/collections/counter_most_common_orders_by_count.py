# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_most_common_orders_by_count"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: Counter.most_common(n) returns the n highest-count (element, count) pairs in descending-count order"""
from collections import Counter

assert Counter([1, 2, 2, 3, 3, 3]).most_common(1) == [(3, 3)], "single most common"
assert Counter("banana").most_common(2) == [("a", 3), ("n", 2)], "top two by count"
assert Counter("abracadabra").most_common(3) == [("a", 5), ("b", 2), ("r", 2)], "top three by count"

print("counter_most_common_orders_by_count OK")
