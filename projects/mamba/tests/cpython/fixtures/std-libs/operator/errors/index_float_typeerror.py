# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "index_float_typeerror"
# subject = "operator.index"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.index: index_float_typeerror (errors)."""
import operator

_raised = False
try:
    operator.index(1.5)
except TypeError:
    _raised = True
assert _raised, "index_float_typeerror: expected TypeError"
print("index_float_typeerror OK")
