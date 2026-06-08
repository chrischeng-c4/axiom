# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "setswitchinterval_str_raises"
# subject = "sys.setswitchinterval"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.setswitchinterval: setswitchinterval_str_raises (errors)."""
import sys

_raised = False
try:
    sys.setswitchinterval('a')
except TypeError:
    _raised = True
assert _raised, "setswitchinterval_str_raises: expected TypeError"
print("setswitchinterval_str_raises OK")
