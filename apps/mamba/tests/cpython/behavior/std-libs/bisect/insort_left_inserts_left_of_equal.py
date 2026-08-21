# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "insort_left_inserts_left_of_equal"
# subject = "bisect.insort_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.insort_left: insort_left keeps the list sorted and inserts left of an equal run"""
import bisect

c = [1, 3, 3, 5]
bisect.insort_left(c, 3)
assert c == [1, 3, 3, 3, 5], f"insort_left sorted = {c!r}"

print("insort_left_inserts_left_of_equal OK")
