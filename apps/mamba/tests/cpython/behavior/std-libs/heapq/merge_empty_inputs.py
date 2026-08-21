# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "merge_empty_inputs"
# subject = "heapq.merge"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.merge: merge() of empty inputs (and of no inputs, and with key=) yields nothing"""
import heapq

assert list(heapq.merge([], [])) == [], "merge of empties is empty"
assert list(heapq.merge([], [], key=len)) == [], "merge of empties with key="
assert list(heapq.merge()) == [], "merge of no inputs is empty"
print("merge_empty_inputs OK")
