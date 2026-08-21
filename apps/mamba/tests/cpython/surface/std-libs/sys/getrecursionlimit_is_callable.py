# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "getrecursionlimit_is_callable"
# subject = "sys.getrecursionlimit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.getrecursionlimit: getrecursionlimit_is_callable (surface)."""
import sys

assert callable(sys.getrecursionlimit)
print("getrecursionlimit_is_callable OK")
