# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "truediv_is_callable"
# subject = "operator.truediv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.truediv: truediv_is_callable (surface)."""
import operator

assert callable(operator.truediv)
print("truediv_is_callable OK")
