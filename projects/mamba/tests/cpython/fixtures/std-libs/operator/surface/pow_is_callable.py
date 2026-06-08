# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "pow_is_callable"
# subject = "operator.pow"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.pow: pow_is_callable (surface)."""
import operator

assert callable(operator.pow)
print("pow_is_callable OK")
