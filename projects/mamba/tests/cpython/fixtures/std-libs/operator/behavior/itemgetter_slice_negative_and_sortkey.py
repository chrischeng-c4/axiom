# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "itemgetter_slice_negative_and_sortkey"
# subject = "operator.itemgetter"
# kind = "semantic"
# xfail = "operator.itemgetter(i)(row) returns 0 instead of row[i] under mamba (repo-memory project_mamba_operator_itemgetter_returns_zero)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.itemgetter: itemgetter accepts negative indices and slice objects as keys and works as a map/sort key over a list of records"""
import operator

text = "ABCDE"
assert operator.itemgetter(-1)(tuple(text)) == "E", "negative index"
assert operator.itemgetter(slice(2, 4))(tuple(text)) == ("C", "D"), "slice key"
assert operator.itemgetter(0)(range(100, 200)) == 100, "range"

inventory = [("apple", 3), ("banana", 2), ("pear", 5), ("orange", 1)]
by_count = operator.itemgetter(1)
assert list(map(by_count, inventory)) == [3, 2, 5, 1], "map key"
assert sorted(inventory, key=by_count) == [
    ("orange", 1), ("banana", 2), ("apple", 3), ("pear", 5)
], "sort key"

print("itemgetter_slice_negative_and_sortkey OK")
