# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "surface"
# case = "insort_is_callable"
# subject = "bisect.insort"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.insort: insort_is_callable (surface)."""
import bisect

assert callable(bisect.insort)
print("insort_is_callable OK")
