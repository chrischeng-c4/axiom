# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "le_is_callable"
# subject = "operator.le"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.le: le_is_callable (surface)."""
import operator

assert callable(operator.le)
print("le_is_callable OK")
