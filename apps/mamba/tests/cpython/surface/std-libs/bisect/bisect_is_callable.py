# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "surface"
# case = "bisect_is_callable"
# subject = "bisect.bisect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect: bisect_is_callable (surface)."""
import bisect

assert callable(bisect.bisect)
print("bisect_is_callable OK")
