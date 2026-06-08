# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "gt_is_callable"
# subject = "operator.gt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.gt: gt_is_callable (surface)."""
import operator

assert callable(operator.gt)
print("gt_is_callable OK")
