# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seed_does_not_mutate_bytearray_buffer"
# subject = "random.Random.seed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.seed: seeding from a bytearray must not mutate the caller's buffer (bug 44018): seed(bytearray(b'1234')) leaves the bytearray unchanged"""
import random

gen = random.Random()

# Seeding from a bytearray must not mutate the caller's buffer (bug 44018).
buf = bytearray(b"1234")
gen.seed(buf)
assert buf == bytearray(b"1234"), f"seed mutated buffer: {buf!r}"

print("seed_does_not_mutate_bytearray_buffer OK")
