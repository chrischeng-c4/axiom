# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "itemgetter_is_callable"
# subject = "operator.itemgetter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.itemgetter: itemgetter_is_callable (surface)."""
import operator

assert callable(operator.itemgetter)
print("itemgetter_is_callable OK")
