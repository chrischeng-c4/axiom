# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "lshift_is_callable"
# subject = "operator.lshift"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.lshift: lshift_is_callable (surface)."""
import operator

assert callable(operator.lshift)
print("lshift_is_callable OK")
