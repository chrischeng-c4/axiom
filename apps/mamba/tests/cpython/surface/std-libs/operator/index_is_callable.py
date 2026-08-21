# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "index_is_callable"
# subject = "operator.index"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.index: index_is_callable (surface)."""
import operator

assert callable(operator.index)
print("index_is_callable OK")
