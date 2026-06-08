# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "setrecursionlimit_negative_raises"
# subject = "sys.setrecursionlimit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.setrecursionlimit: setrecursionlimit_negative_raises (errors)."""
import sys

_raised = False
try:
    sys.setrecursionlimit(-5)
except ValueError:
    _raised = True
assert _raised, "setrecursionlimit_negative_raises: expected ValueError"
print("setrecursionlimit_negative_raises OK")
