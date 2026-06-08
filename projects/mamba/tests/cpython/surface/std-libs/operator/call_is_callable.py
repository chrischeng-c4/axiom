# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "call_is_callable"
# subject = "operator.call"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.call: call_is_callable (surface)."""
import operator

assert callable(operator.call)
print("call_is_callable OK")
