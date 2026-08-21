# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "pairwise_consecutive_pairs"
# subject = "itertools.pairwise"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.pairwise: pairwise yields consecutive (prev, curr) 2-tuples; fewer than two inputs yields nothing"""
import itertools

assert list(itertools.pairwise([1, 2, 3, 4])) == [(1, 2), (2, 3), (3, 4)], "pairwise list"
assert list(itertools.pairwise("ABCDE")) == [("A", "B"), ("B", "C"), ("C", "D"), ("D", "E")], "pairwise string"
assert list(itertools.pairwise([1])) == [], "pairwise single"
assert list(itertools.pairwise([])) == [], "pairwise empty"

print("pairwise_consecutive_pairs OK")
