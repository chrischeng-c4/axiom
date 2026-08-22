# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seeded_sample_heap_integer_range_matches_cpython"
# subject = "random.Random.sample"
# kind = "semantic"
# module = "random"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.sample preserves seeded parity for a large integer range."""
import random


rng = random.Random(1234)
base = 1 << 60
assert rng.sample(range(base, base + 22), 5) == [
    1152921504606846990,
    1152921504606846979,
    1152921504606846976,
    1152921504606846978,
    1152921504606846994,
]
assert rng.getrandbits(32) == 150006740

print("seeded_sample_heap_integer_range_matches_cpython OK")
