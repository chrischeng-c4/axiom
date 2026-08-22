# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seeded_sample_unicode_string_matches_cpython"
# subject = "random.Random.sample"
# kind = "semantic"
# module = "random"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.sample preserves seeded parity for a Unicode string population."""
import random


rng = random.Random(1234)
assert rng.sample("aβcδ", 3) == ["δ", "a", "c"]
assert rng.getrandbits(32) == 389311301

print("seeded_sample_unicode_string_matches_cpython OK")
