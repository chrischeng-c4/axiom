# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "setswitchinterval_zero_raises"
# subject = "sys.setswitchinterval"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.setswitchinterval: setswitchinterval_zero_raises (errors)."""
import sys

_raised = False
try:
    sys.setswitchinterval(0.0)
except ValueError:
    _raised = True
assert _raised, "setswitchinterval_zero_raises: expected ValueError"
print("setswitchinterval_zero_raises OK")
