# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "permutations_r_orderings"
# subject = "itertools.permutations"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.permutations: permutations(it, r) yields ordered r-length arrangements; default r==n; r==1 is singletons; len is n!/(n-r)!"""
import itertools

perms = list(itertools.permutations([1, 2, 3], 2))
assert len(perms) == 6, f"permutations len = {len(perms)!r}"
assert (1, 2) in perms, "perm (1,2)"
assert list(itertools.permutations([1, 2])) == [(1, 2), (2, 1)], "default r==n"
assert list(itertools.permutations([1, 2, 3], 1)) == [(1,), (2,), (3,)], "r==1 singletons"

print("permutations_r_orderings OK")
