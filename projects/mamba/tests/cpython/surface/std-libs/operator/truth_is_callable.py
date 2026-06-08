# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "truth_is_callable"
# subject = "operator.truth"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.truth: truth_is_callable (surface)."""
import operator

assert callable(operator.truth)
print("truth_is_callable OK")
