# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "getitem_is_callable"
# subject = "operator.getitem"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.getitem: getitem_is_callable (surface)."""
import operator

assert callable(operator.getitem)
print("getitem_is_callable OK")
