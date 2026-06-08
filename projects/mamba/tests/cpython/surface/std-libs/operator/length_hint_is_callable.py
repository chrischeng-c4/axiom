# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "length_hint_is_callable"
# subject = "operator.length_hint"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.length_hint: length_hint_is_callable (surface)."""
import operator

assert callable(operator.length_hint)
print("length_hint_is_callable OK")
