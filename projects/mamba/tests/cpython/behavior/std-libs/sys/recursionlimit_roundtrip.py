# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "recursionlimit_roundtrip"
# subject = "sys.setrecursionlimit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.setrecursionlimit: setrecursionlimit(500) is observable via getrecursionlimit(), then restored to the original limit"""
import sys

_orig = sys.getrecursionlimit()
sys.setrecursionlimit(500)
assert sys.getrecursionlimit() == 500, f"set to 500: {sys.getrecursionlimit()!r}"
sys.setrecursionlimit(_orig)  # restore
assert sys.getrecursionlimit() == _orig, f"restored: {sys.getrecursionlimit()!r}"
print("recursionlimit_roundtrip OK")
