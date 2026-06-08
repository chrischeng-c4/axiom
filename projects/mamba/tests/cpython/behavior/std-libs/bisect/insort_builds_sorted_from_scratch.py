# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "insort_builds_sorted_from_scratch"
# subject = "bisect.insort"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.insort: repeated insort on an initially-empty list builds a fully sorted list"""
import bisect

f = []
for v in [5, 3, 1, 4, 2]:
    bisect.insort(f, v)
assert f == [1, 2, 3, 4, 5], f"insort sort = {f!r}"

print("insort_builds_sorted_from_scratch OK")
