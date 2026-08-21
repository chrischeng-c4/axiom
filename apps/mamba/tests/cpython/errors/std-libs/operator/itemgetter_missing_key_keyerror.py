# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "itemgetter_missing_key_keyerror"
# subject = "operator.itemgetter"
# kind = "mechanical"
# xfail = "operator.itemgetter(i)(row) returns 0 and swallows the raise under mamba (repo-memory project_mamba_operator_itemgetter_returns_zero)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.itemgetter: itemgetter_missing_key_keyerror (errors)."""
import operator

_raised = False
try:
    operator.itemgetter("a")({})
except KeyError:
    _raised = True
assert _raised, "itemgetter_missing_key_keyerror: expected KeyError"
print("itemgetter_missing_key_keyerror OK")
