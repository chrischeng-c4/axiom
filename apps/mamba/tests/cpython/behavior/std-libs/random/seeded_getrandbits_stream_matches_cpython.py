# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seeded_getrandbits_stream_matches_cpython"
# subject = "random.Random.getrandbits"
# kind = "semantic"
# module = "random"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.getrandbits: exact CPython MT word consumption and assembly."""
import random

rng = random.Random(1234)
values = [rng.getrandbits(width) for width in (1, 8, 16, 32, 64)]

assert all(type(value) is int for value in values), values
assert values == [1, 199, 28883, 501869158, 1672079305790387732]

print("seeded_getrandbits_stream_matches_cpython OK")
