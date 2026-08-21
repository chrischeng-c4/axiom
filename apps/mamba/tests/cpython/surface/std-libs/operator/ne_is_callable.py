# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "ne_is_callable"
# subject = "operator.ne"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.ne: ne_is_callable (surface)."""
import operator

assert callable(operator.ne)
print("ne_is_callable OK")
