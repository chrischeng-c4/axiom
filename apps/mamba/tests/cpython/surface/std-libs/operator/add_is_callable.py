# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "add_is_callable"
# subject = "operator.add"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.add: add_is_callable (surface)."""
import operator

assert callable(operator.add)
print("add_is_callable OK")
