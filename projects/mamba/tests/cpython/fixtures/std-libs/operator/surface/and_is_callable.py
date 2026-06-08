# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "and_is_callable"
# subject = "operator.and_"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.and_: and_is_callable (surface)."""
import operator

assert callable(operator.and_)
print("and_is_callable OK")
