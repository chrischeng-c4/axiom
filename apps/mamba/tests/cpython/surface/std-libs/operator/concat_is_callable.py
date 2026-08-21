# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "concat_is_callable"
# subject = "operator.concat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.concat: concat_is_callable (surface)."""
import operator

assert callable(operator.concat)
print("concat_is_callable OK")
