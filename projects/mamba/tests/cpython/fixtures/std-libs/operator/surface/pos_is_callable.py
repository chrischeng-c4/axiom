# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "pos_is_callable"
# subject = "operator.pos"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.pos: pos_is_callable (surface)."""
import operator

assert callable(operator.pos)
print("pos_is_callable OK")
