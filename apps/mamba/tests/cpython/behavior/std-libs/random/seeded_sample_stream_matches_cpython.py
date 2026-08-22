# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seeded_sample_stream_matches_cpython"
# subject = "random.Random.sample"
# kind = "semantic"
# module = "random"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.sample: pool and selected-set paths match CPython streams."""
import random


rng = random.Random(1234)
assert rng.sample(range(10), 4) == [7, 1, 0, 9]
assert rng.sample(range(10), 4) == [9, 0, 1, 8]
assert rng.getrandbits(32) == 3292010550

selected_rng = random.Random(1234)
assert selected_rng.sample(range(22), 5) == [14, 3, 0, 2, 18]
assert selected_rng.getrandbits(32) == 150006740

print("seeded_sample_stream_matches_cpython OK")
