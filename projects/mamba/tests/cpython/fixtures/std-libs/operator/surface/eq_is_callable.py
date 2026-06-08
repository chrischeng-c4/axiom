# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "eq_is_callable"
# subject = "operator.eq"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.eq: eq_is_callable (surface)."""
import operator

assert callable(operator.eq)
print("eq_is_callable OK")
