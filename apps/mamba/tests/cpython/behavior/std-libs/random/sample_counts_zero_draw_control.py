# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "sample_counts_zero_draw_control"
# subject = "random.Random.sample"
# kind = "semantic"
# module = "random"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.sample counts accepts a zero-draw request without consuming RNG state."""
import random


rng = random.Random(1234)
assert rng.sample("aβcδ", 0, counts=[2, 1, 3, 2]) == []
assert rng.getrandbits(32) == 4150886329

print("sample_counts_zero_draw_control OK")
