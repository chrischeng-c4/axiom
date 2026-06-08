# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "xor_is_callable"
# subject = "operator.xor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.xor: xor_is_callable (surface)."""
import operator

assert callable(operator.xor)
print("xor_is_callable OK")
