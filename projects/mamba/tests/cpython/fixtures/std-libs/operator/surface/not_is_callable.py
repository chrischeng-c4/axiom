# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "not_is_callable"
# subject = "operator.not_"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.not_: not_is_callable (surface)."""
import operator

assert callable(operator.not_)
print("not_is_callable OK")
