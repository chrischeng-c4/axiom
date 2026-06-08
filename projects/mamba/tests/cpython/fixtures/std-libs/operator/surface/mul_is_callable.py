# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "mul_is_callable"
# subject = "operator.mul"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.mul: mul_is_callable (surface)."""
import operator

assert callable(operator.mul)
print("mul_is_callable OK")
