# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "module_all_subset_of_dir"
# subject = "random"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random: everything advertised in random.__all__ is actually exported by the module (__all__ is a subset of dir(random)) and the documented constructors/helpers (Random, SystemRandom, random, randint, choice, sample, shuffle, choices, getrandbits, randbytes, seed) all exist"""
import random

import math

# Everything advertised in __all__ is actually exported by the module.
assert set(random.__all__) <= set(dir(random)), "__all__ not a subset of dir"

# The documented constructors and helpers exist on the module.
for name in ("Random", "SystemRandom", "random", "randint", "choice", "sample",
             "shuffle", "choices", "getrandbits", "randbytes", "seed"):
    assert hasattr(random, name), f"missing random.{name}"

# TWOPI constant matches 2*pi within float tolerance.
assert abs(random.TWOPI - 2 * math.pi) < 1e-12, f"TWOPI = {random.TWOPI!r}"

print("module_all_subset_of_dir OK")
