# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "setrecursionlimit_zero_raises"
# subject = "sys.setrecursionlimit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.setrecursionlimit: setrecursionlimit_zero_raises (errors)."""
import sys

_raised = False
try:
    sys.setrecursionlimit(0)
except ValueError:
    _raised = True
assert _raised, "setrecursionlimit_zero_raises: expected ValueError"
print("setrecursionlimit_zero_raises OK")
