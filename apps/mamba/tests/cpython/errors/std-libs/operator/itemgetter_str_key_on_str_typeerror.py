# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "itemgetter_str_key_on_str_typeerror"
# subject = "operator.itemgetter"
# kind = "mechanical"
# xfail = "operator.itemgetter(i)(row) returns 0 and swallows the raise under mamba (repo-memory project_mamba_operator_itemgetter_returns_zero)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.itemgetter: itemgetter_str_key_on_str_typeerror (errors)."""
import operator

_raised = False
try:
    operator.itemgetter("name")("ABCDE")
except TypeError:
    _raised = True
assert _raised, "itemgetter_str_key_on_str_typeerror: expected TypeError"
print("itemgetter_str_key_on_str_typeerror OK")
