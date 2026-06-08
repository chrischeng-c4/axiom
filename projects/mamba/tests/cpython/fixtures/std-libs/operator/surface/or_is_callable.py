# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "or_is_callable"
# subject = "operator.or_"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.or_: or_is_callable (surface)."""
import operator

assert callable(operator.or_)
print("or_is_callable OK")
