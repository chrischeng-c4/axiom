# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "combinations_r_subsets"
# subject = "itertools.combinations"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.combinations: combinations(it, r) yields sorted r-subsets without repetition; r==0 is [()], r==n is [tuple(it)]"""
import itertools

combs = list(itertools.combinations([1, 2, 3], 2))
assert combs == [(1, 2), (1, 3), (2, 3)], f"combinations = {combs!r}"
assert list(itertools.combinations([1, 2, 3], 0)) == [()], "r==0 is [()]"
assert list(itertools.combinations([1, 2, 3], 3)) == [(1, 2, 3)], "r==n is the whole tuple"
assert list(itertools.combinations("abc", 2)) == [("a", "b"), ("a", "c"), ("b", "c")], "string combinations"

print("combinations_r_subsets OK")
