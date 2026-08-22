# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "sample_rejects_reversed_iterator_without_draw"
# subject = "random.Random.sample"
# kind = "semantic"
# module = "random"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""Random.sample rejects a reversed iterator before consuming its RNG stream."""
import random
from typing import Any


rng = random.Random(1234)
bad_population: Any = reversed(("left", "right"))

try:
    rng.sample(bad_population, 1)
except TypeError:
    pass
else:
    raise AssertionError("sample(reversed_tuple, 1) returned normally")

# The sequence-wall TypeError must happen before any random draw.
assert rng.getrandbits(32) == 4150886329

print("sample_rejects_reversed_iterator_without_draw OK")
