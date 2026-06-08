# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "seed_rejects_unhashable_types"
# subject = "random.Random.seed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.seed: unhashable / unsupported seed types raise TypeError: complex, tuple, list, and dict seeds all raise; passing too many positional args also raises TypeError"""
import random

gen = random.Random()

# Unhashable / unsupported seed types raise TypeError.
for arg in [1 + 2j, ("a", "b", "c"), [1, 2, 3], {"one": 1}]:
    try:
        gen.seed(arg)
        raise AssertionError(f"expected TypeError for seed {arg!r}")
    except TypeError:
        pass

# seed accepts at most one positional argument plus version.
try:
    gen.seed(1, 2, 3, 4)
    raise AssertionError("expected TypeError for too many args")
except TypeError:
    pass

print("seed_rejects_unhashable_types OK")
