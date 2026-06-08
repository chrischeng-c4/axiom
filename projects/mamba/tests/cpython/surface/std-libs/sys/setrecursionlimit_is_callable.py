# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "setrecursionlimit_is_callable"
# subject = "sys.setrecursionlimit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.setrecursionlimit: setrecursionlimit_is_callable (surface)."""
import sys

assert callable(sys.setrecursionlimit)
print("setrecursionlimit_is_callable OK")
