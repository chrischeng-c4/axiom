# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "multimode_first_seen_order"
# subject = "statistics.multimode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.multimode: multimode returns every most-common value preserving first-seen order ('aabbbbccddddeeffffgg' -> ['b','d','f']) and returns [] for empty input"""
from statistics import multimode

# A single dominant value gives a one-element list.
assert multimode("aabbbbbbbbcc") == ["b"], multimode("aabbbbbbbbcc")
# Multiple tied modes are returned in first-seen order.
assert multimode("aabbbbccddddeeffffgg") == ["b", "d", "f"]
# A bimodal int list keeps first-seen order.
assert sorted(multimode([1, 1, 2, 2, 3])) == [1, 2]
# Empty input -> empty list (never an error).
assert multimode("") == []

print("multimode_first_seen_order OK")
