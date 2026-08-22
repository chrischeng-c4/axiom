# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "random_error_helpers_do_not_consume_rng"
# subject = "random.Random.sample"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""Random error paths preserve the seeded stream where CPython does."""
import random
from typing import Any


rng = random.Random(1234)

bad_population: Any = 1
try:
    rng.sample(bad_population, 1)
except TypeError:
    pass
else:
    raise AssertionError("sample(bad_population, 1) returned normally")

try:
    rng.sample([], 1)
except ValueError:
    pass
else:
    raise AssertionError("sample([], 1) returned normally")

try:
    rng.choice([])
except IndexError:
    pass
else:
    raise AssertionError("choice([]) returned normally")

assert rng.getrandbits(32) == 4150886329

print("random_error_helpers_do_not_consume_rng OK")
