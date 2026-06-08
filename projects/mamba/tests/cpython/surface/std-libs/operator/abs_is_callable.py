# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "abs_is_callable"
# subject = "operator.abs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.abs: abs_is_callable (surface)."""
import operator

assert callable(operator.abs)
print("abs_is_callable OK")
