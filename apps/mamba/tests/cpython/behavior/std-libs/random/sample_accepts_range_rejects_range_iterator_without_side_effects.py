# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "sample_accepts_range_rejects_range_iterator_without_side_effects"
# subject = "random.sample"
# kind = "semantic"
# module = "random"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.sample keeps range sequences accepted and rejects their iterators."""
import random
from typing import Any


random.seed(1234)
assert random.sample(range(5), 2) == [3, 0]

rng = random.Random(1234)
population: Any = iter(range(5))
try:
    rng.sample(population, 2)
except TypeError:
    pass
else:
    raise AssertionError("sample(iter(range(5)), 2) returned normally")

assert next(population) == 0
assert rng.getrandbits(32) == 4150886329

print("sample_accepts_range_rejects_range_iterator_without_side_effects OK")
