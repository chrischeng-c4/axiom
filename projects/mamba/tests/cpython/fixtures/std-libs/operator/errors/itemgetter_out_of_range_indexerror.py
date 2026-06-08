# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "itemgetter_out_of_range_indexerror"
# subject = "operator.itemgetter"
# kind = "mechanical"
# xfail = "operator.itemgetter(i)(row) returns 0 and swallows the raise under mamba (repo-memory project_mamba_operator_itemgetter_returns_zero)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.itemgetter: itemgetter_out_of_range_indexerror (errors)."""
import operator

_raised = False
try:
    operator.itemgetter(5)([1, 2, 3])
except IndexError:
    _raised = True
assert _raised, "itemgetter_out_of_range_indexerror: expected IndexError"
print("itemgetter_out_of_range_indexerror OK")
