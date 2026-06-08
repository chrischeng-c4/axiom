# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "insort_honors_key"
# subject = "bisect.insort_right"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
"""bisect.insort_right: insort_right honors key= and keeps the list sorted by the projected key"""
import bisect

rows = [("a", 1), ("b", 3), ("c", 5)]
bisect.insort_right(rows, ("x", 4), key=lambda r: r[1])
assert rows == [("a", 1), ("b", 3), ("x", 4), ("c", 5)], f"insort key= = {rows!r}"

print("insort_honors_key OK")
