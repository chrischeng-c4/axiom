# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seeded_choices_unicode_string_matches_cpython"
# subject = "random.Random.choices"
# kind = "semantic"
# module = "random"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.choices preserves seeded Unicode-string stream parity."""
import random


rng = random.Random(1234)
assert rng.choices("aβcδ", k=6) == ["δ", "β", "a", "δ", "δ", "c"]
assert rng.getrandbits(32) == 2884343186

print("seeded_choices_unicode_string_matches_cpython OK")
