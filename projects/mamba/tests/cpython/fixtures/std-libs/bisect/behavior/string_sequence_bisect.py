# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "string_sequence_bisect"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: bisect works on any comparable sorted sequence, e.g. a sorted list of strings"""
import bisect

words = ["apple", "cherry", "mango", "orange"]
pos = bisect.bisect_left(words, "grape")
assert pos == 2, f"string bisect = {pos!r}"

print("string_sequence_bisect OK")
