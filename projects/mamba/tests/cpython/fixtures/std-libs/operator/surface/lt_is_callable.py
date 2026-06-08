# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "lt_is_callable"
# subject = "operator.lt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.lt: lt_is_callable (surface)."""
import operator

assert callable(operator.lt)
print("lt_is_callable OK")
