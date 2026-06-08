# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "surface"
# case = "bisect_left_is_callable"
# subject = "bisect.bisect_left"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: bisect_left_is_callable (surface)."""
import bisect

assert callable(bisect.bisect_left)
print("bisect_left_is_callable OK")
