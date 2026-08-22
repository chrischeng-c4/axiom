# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# module = "random"
# dimension = "behavior"
# case = "seeded_negative_randrange_stream_matches_cpython"
# subject = "random.Random.randrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.randrange: negative-step ceiling and rejection consumption."""
import random


first = random.Random(1234)
assert [first.randrange(10, -10, -3) for _ in range(8)] == [
    -8,
    1,
    10,
    10,
    10,
    -8,
    -2,
    10,
]
assert first.getrandbits(32) == 2884343186

second = random.Random(1234)
assert [second.randrange(10, 0, -1) for _ in range(8)] == [
    3,
    9,
    10,
    9,
    1,
    10,
    9,
    9,
]
assert second.getrandbits(32) == 3292010550

print("seeded_negative_randrange_stream_matches_cpython OK")
