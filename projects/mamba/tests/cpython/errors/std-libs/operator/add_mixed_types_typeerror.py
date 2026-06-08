# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "add_mixed_types_typeerror"
# subject = "operator.add"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.add: add_mixed_types_typeerror (errors)."""
import operator

_raised = False
try:
    operator.add(1, "a")
except TypeError:
    _raised = True
assert _raised, "add_mixed_types_typeerror: expected TypeError"
print("add_mixed_types_typeerror OK")
