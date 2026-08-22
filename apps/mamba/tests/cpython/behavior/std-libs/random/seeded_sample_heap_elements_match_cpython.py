# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seeded_sample_heap_elements_match_cpython"
# subject = "random.Random.sample"
# kind = "semantic"
# module = "random"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.sample preserves heap-backed temporary list element parity."""
import random


rng = random.Random(1234)
picked = rng.sample(list("abcdef"), 3)
assert picked == ["d", "a", "e"]
assert rng.getrandbits(32) == 389311301

print("seeded_sample_heap_elements_match_cpython OK")
