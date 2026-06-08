# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "surface"
# case = "insort_right_is_callable"
# subject = "bisect.insort_right"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.insort_right: insort_right_is_callable (surface)."""
import bisect

assert callable(bisect.insort_right)
print("insort_right_is_callable OK")
