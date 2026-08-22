# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seeded_int_random_stream_matches_cpython"
# subject = "random.Random.random"
# kind = "semantic"
# module = "random"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.random: exact CPython 3.12 integer-seed MT stream parity for 0, 42, 1234, -1, 2**40, 10**20, and the 625-word boundary seed"""
import random

long_seed = (1 << (32 * 624)) + 1
assert random.Random(long_seed).random() == 0.20803343434130195

rng = random.Random(0)
assert [rng.random() for _ in range(4)] == [
    0.8444218515250481,
    0.7579544029403025,
    0.420571580830845,
    0.25891675029296335,
]

rng = random.Random(42)
assert [rng.random() for _ in range(4)] == [
    0.6394267984578837,
    0.025010755222666936,
    0.27502931836911926,
    0.22321073814882275,
]

rng = random.Random(1234)
assert [rng.random() for _ in range(4)] == [
    0.9664535356921388,
    0.4407325991753527,
    0.007491470058587191,
    0.9109759624491242,
]

rng = random.Random(-1)
assert [rng.random() for _ in range(4)] == [
    0.13436424411240122,
    0.8474337369372327,
    0.763774618976614,
    0.2550690257394217,
]

assert random.Random(2**40).random() == 0.1036394562812375
assert random.Random(10**20).random() == 0.018195028987988637

print("seeded_int_random_stream_matches_cpython OK")
