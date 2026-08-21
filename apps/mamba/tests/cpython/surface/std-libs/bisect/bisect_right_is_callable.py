# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "surface"
# case = "bisect_right_is_callable"
# subject = "bisect.bisect_right"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_right: bisect_right_is_callable (surface)."""
import bisect

assert callable(bisect.bisect_right)
print("bisect_right_is_callable OK")
