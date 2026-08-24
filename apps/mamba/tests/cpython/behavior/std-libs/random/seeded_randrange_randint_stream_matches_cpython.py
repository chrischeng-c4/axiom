# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# module = "random"
# dimension = "behavior"
# case = "seeded_randrange_randint_stream_matches_cpython"
# subject = "random.Random.randrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random randint and randrange streams match CPython."""
import random


randint_rng = random.Random(1234)
assert [randint_rng.randint(1, 100) for _ in range(6)] == [100, 57, 15, 1, 12, 75]

stepped_rng = random.Random(1234)
assert [stepped_rng.randrange(10, 100, 3) for _ in range(6)] == [
    82,
    52,
    19,
    10,
    16,
    97,
]

even_rng = random.Random(1234)
even_values = [even_rng.randrange(0, 5, 2) for _ in range(20)]
assert even_values == [2, 0, 0, 0, 4, 0, 4, 4, 0, 0, 2, 0, 0, 0, 0, 2, 4, 4, 2, 4]
assert set(even_values) == {0, 2, 4}

print("seeded_randrange_randint_stream_matches_cpython OK")
