# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "sub_is_callable"
# subject = "operator.sub"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.sub: sub_is_callable (surface)."""
import operator

assert callable(operator.sub)
print("sub_is_callable OK")
