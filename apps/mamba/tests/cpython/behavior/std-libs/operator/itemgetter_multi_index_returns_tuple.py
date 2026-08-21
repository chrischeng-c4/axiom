# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "itemgetter_multi_index_returns_tuple"
# subject = "operator.itemgetter"
# kind = "semantic"
# xfail = "operator.itemgetter(i)(row) returns 0 instead of row[i] under mamba (repo-memory project_mamba_operator_itemgetter_returns_zero)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.itemgetter: a multi-key itemgetter returns a tuple of the indexed elements in argument order"""
import operator

_result = operator.itemgetter(0, 2, 4)([10, 20, 30, 40, 50])
assert _result == (10, 30, 50), f"multi-itemgetter = {_result!r}"

print("itemgetter_multi_index_returns_tuple OK")
