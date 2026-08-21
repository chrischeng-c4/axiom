# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "neg_is_callable"
# subject = "operator.neg"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.neg: neg_is_callable (surface)."""
import operator

assert callable(operator.neg)
print("neg_is_callable OK")
