# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "floordiv_is_callable"
# subject = "operator.floordiv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.floordiv: floordiv_is_callable (surface)."""
import operator

assert callable(operator.floordiv)
print("floordiv_is_callable OK")
