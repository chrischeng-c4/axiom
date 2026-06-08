# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "rshift_is_callable"
# subject = "operator.rshift"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.rshift: rshift_is_callable (surface)."""
import operator

assert callable(operator.rshift)
print("rshift_is_callable OK")
