# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "insort_right_inserts_right_of_equal"
# subject = "bisect.insort_right"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.insort_right: insort_right keeps the list sorted and inserts right of an equal run"""
import bisect

d = [1, 3, 3, 5]
bisect.insort_right(d, 3)
assert d == [1, 3, 3, 3, 5], f"insort_right sorted = {d!r}"

print("insort_right_inserts_right_of_equal OK")
