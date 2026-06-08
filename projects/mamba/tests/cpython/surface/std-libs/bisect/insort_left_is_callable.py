# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "surface"
# case = "insort_left_is_callable"
# subject = "bisect.insort_left"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.insort_left: insort_left_is_callable (surface)."""
import bisect

assert callable(bisect.insort_left)
print("insort_left_is_callable OK")
