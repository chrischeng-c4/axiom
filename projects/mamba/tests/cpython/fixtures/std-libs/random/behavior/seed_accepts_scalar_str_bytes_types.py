# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seed_accepts_scalar_str_bytes_types"
# subject = "random.Random.seed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.seed: Random().seed accepts None / int (incl. huge & negative) / bool / float / str / bytes seeds without raising, and a bytes seed reproduces the same first draw"""
import random

gen = random.Random()

# Hashable scalars and byte strings are all valid seeds.
for arg in [None, 0, 1, -1, 10 ** 20, -10 ** 20, False, True, 3.14, "a", b"xy"]:
    gen.seed(arg)

# Same seed -> same first draw, confirming the seed actually took effect.
gen.seed(b"xy")
first = gen.random()
gen.seed(b"xy")
assert gen.random() == first, "bytes seed not reproducible"

print("seed_accepts_scalar_str_bytes_types OK")
