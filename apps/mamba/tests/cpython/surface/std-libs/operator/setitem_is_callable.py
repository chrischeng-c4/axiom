# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "setitem_is_callable"
# subject = "operator.setitem"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.setitem: setitem_is_callable (surface)."""
import operator

assert callable(operator.setitem)
print("setitem_is_callable OK")
