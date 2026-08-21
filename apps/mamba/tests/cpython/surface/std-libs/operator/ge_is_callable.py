# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "ge_is_callable"
# subject = "operator.ge"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.ge: ge_is_callable (surface)."""
import operator

assert callable(operator.ge)
print("ge_is_callable OK")
