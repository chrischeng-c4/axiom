# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seed_makes_stream_reproducible"
# subject = "random.seed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.seed: re-seeding the module RNG with the same value (99) reproduces the same randint sequence; two seeded runs of [randint(0,100) for _ in range(5)] are equal"""
import random

random.seed(99)
a = [random.randint(0, 100) for _ in range(5)]
random.seed(99)
b = [random.randint(0, 100) for _ in range(5)]
assert a == b, f"seed reproducible: {a!r} != {b!r}"

print("seed_makes_stream_reproducible OK")
