# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "delitem_is_callable"
# subject = "operator.delitem"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.delitem: delitem_is_callable (surface)."""
import operator

assert callable(operator.delitem)
print("delitem_is_callable OK")
