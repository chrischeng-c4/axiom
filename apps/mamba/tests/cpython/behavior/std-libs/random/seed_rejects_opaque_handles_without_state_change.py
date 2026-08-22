# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seed_rejects_opaque_handles_without_state_change"
# subject = "random.Random.seed"
# kind = "semantic"
# module = "random"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random rejects unsupported seed objects without changing state."""
import random
from typing import Any


unsupported_object: Any = object()
try:
    random.Random(unsupported_object)
except TypeError:
    pass
else:
    raise AssertionError("Random(object()) returned normally")

unsupported_range: Any = range(3)
try:
    random.Random(unsupported_range)
except TypeError:
    pass
else:
    raise AssertionError("Random(range(3)) returned normally")

bad_iterator: Any = iter(range(3))
try:
    random.Random(bad_iterator)
except TypeError:
    pass
else:
    raise AssertionError("Random(iter(range(3))) returned normally")

rng = random.Random(1234)
bad_range: Any = range(3)
try:
    rng.seed(bad_range)
except TypeError:
    pass
else:
    raise AssertionError("seed(range(3)) returned normally")
assert rng.getrandbits(32) == 4150886329

rng = random.Random(1234)
bad_iterator = iter(range(3))
try:
    rng.seed(bad_iterator)
except TypeError:
    pass
else:
    raise AssertionError("seed(iter(range(3))) returned normally")
assert rng.getrandbits(32) == 4150886329
assert next(bad_iterator) == 0

rng = random.Random()
rng.seed(1 << 200)
assert rng.getrandbits(32) == 2444605360

print("seed_rejects_opaque_handles_without_state_change OK")
