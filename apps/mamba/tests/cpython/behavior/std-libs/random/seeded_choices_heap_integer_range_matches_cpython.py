# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seeded_choices_heap_integer_range_matches_cpython"
# subject = "random.Random.choices"
# kind = "semantic"
# module = "random"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.choices preserves seeded large-integer range parity."""
import random


rng = random.Random(1234)
base = 1 << 60
assert rng.choices(range(base, base + 4), k=6) == [
    1152921504606846979,
    1152921504606846977,
    1152921504606846976,
    1152921504606846979,
    1152921504606846979,
    1152921504606846978,
]
assert rng.getrandbits(32) == 2884343186

print("seeded_choices_heap_integer_range_matches_cpython OK")
