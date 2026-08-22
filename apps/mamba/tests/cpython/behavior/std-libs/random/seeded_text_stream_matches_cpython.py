# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seeded_text_stream_matches_cpython"
# subject = "random.Random.seed"
# kind = "semantic"
# module = "random"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.seed: exact CPython 3.12 version-2 stream parity for text-like abc seeds."""
import random

rng = random.Random("abc")
assert [rng.random() for _ in range(4)] == [
    0.7720246314157545,
    0.5581691373899791,
    0.7095325359498886,
    0.3543265309713739,
]

rng = random.Random(b"abc")
assert [rng.random() for _ in range(4)] == [
    0.7720246314157545,
    0.5581691373899791,
    0.7095325359498886,
    0.3543265309713739,
]

seed = bytearray(b"abc")
rng = random.Random(seed)
assert [rng.random() for _ in range(4)] == [
    0.7720246314157545,
    0.5581691373899791,
    0.7095325359498886,
    0.3543265309713739,
]
assert seed == bytearray(b"abc")

print("seeded_text_stream_matches_cpython OK")
