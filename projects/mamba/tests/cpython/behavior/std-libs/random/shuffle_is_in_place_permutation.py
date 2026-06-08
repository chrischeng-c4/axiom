# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "shuffle_is_in_place_permutation"
# subject = "random.shuffle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.shuffle: shuffle mutates the list in place and is a permutation: after shuffling list(range(5)) the sorted result is still [0,1,2,3,4]"""
import random

random.seed(5)
lst = list(range(5))
random.shuffle(lst)
assert sorted(lst) == [0, 1, 2, 3, 4], f"shuffle preserves elements: {lst!r}"

print("shuffle_is_in_place_permutation OK")
