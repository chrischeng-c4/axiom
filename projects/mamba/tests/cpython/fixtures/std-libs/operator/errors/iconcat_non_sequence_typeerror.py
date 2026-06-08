# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "iconcat_non_sequence_typeerror"
# subject = "operator.iconcat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.iconcat: iconcat_non_sequence_typeerror (errors)."""
import operator

_raised = False
try:
    operator.iconcat(1, 0.5)
except TypeError:
    _raised = True
assert _raised, "iconcat_non_sequence_typeerror: expected TypeError"
print("iconcat_non_sequence_typeerror OK")
