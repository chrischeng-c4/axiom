# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "groupby_sorted_keys_and_counts"
# subject = "itertools.groupby"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.groupby: sort-then-group idiom over a string gives distinct keys and per-group element counts"""
import itertools

s = "abracadabra"
keys = [k for k, g in itertools.groupby(sorted(s))]
assert keys == ["a", "b", "c", "d", "r"], f"keys = {keys!r}"
counts = [(len(list(g)), k) for k, g in itertools.groupby(sorted(s))]
assert counts == [(5, "a"), (2, "b"), (1, "c"), (1, "d"), (2, "r")], f"counts = {counts!r}"

print("groupby_sorted_keys_and_counts OK")
