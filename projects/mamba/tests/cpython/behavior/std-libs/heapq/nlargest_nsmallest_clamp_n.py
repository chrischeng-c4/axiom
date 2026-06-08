# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "nlargest_nsmallest_clamp_n"
# subject = "heapq.nlargest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.nlargest: nlargest/nsmallest clamp n at the boundaries: n=0 returns [] and n>len returns the whole sequence fully sorted"""
import heapq

_seq = [4, 1, 7, 3, 9, 2]
assert heapq.nlargest(0, _seq) == [], "nlargest(0) is empty"
assert heapq.nsmallest(0, _seq) == [], "nsmallest(0) is empty"
assert heapq.nlargest(100, _seq) == sorted(_seq, reverse=True), "nlargest(n>len)"
assert heapq.nsmallest(100, _seq) == sorted(_seq), "nsmallest(n>len)"
print("nlargest_nsmallest_clamp_n OK")
