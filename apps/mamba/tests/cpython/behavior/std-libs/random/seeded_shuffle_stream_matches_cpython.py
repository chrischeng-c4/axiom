# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seeded_shuffle_stream_matches_cpython"
# subject = "random.Random.shuffle"
# kind = "semantic"
# module = "random"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.shuffle: seeded in-place permutation and MT stream match CPython."""
import random


rng = random.Random(1234)
values = list(range(8))
assert rng.shuffle(values) is None
assert values == [1, 3, 2, 4, 5, 6, 0, 7]
assert rng.getrandbits(32) == 422719469

print("seeded_shuffle_stream_matches_cpython OK")
