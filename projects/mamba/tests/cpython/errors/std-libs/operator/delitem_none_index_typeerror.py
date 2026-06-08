# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "delitem_none_index_typeerror"
# subject = "operator.delitem"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.delitem: delitem_none_index_typeerror (errors)."""
import operator

_raised = False
try:
    operator.delitem([1, 2, 3], None)
except TypeError:
    _raised = True
assert _raised, "delitem_none_index_typeerror: expected TypeError"
print("delitem_none_index_typeerror OK")
