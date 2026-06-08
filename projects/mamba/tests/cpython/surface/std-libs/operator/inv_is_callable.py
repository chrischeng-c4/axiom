# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "inv_is_callable"
# subject = "operator.inv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.inv: inv_is_callable (surface)."""
import operator

assert callable(operator.inv)
print("inv_is_callable OK")
