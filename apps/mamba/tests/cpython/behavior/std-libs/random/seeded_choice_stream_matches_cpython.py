# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seeded_choice_stream_matches_cpython"
# subject = "random.Random.choice"
# kind = "semantic"
# module = "random"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.choice: seeded list-only _randbelow stream matches CPython."""
import random


rng = random.Random(1234)
assert [rng.choice(list("abcdef")) for _ in range(6)] == ["d", "a", "a", "a", "e", "a"]
assert rng.getrandbits(32) == 2884343186

print("seeded_choice_stream_matches_cpython OK")
