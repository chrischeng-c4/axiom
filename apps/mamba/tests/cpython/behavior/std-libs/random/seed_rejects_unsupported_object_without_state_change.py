# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seed_rejects_unsupported_object_without_state_change"
# subject = "random.Random.seed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.seed: unsupported user objects are rejected before state mutation."""
import random
from typing import Any

rng = random.Random(1234)
bad_seed: Any = object()

try:
    rng.seed(bad_seed)
except TypeError:
    pass
else:
    raise AssertionError("seed accepted unsupported object")

assert rng.getrandbits(32) == 4150886329
print("seed_rejects_unsupported_object_without_state_change OK")
