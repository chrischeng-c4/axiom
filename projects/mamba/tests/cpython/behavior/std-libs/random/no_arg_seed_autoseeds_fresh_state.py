# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "no_arg_seed_autoseeds_fresh_state"
# subject = "random.Random.seed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.seed: seeding with no argument auto-seeds from an OS entropy source: two consecutive getstate() snapshots after seed() differ"""
import random

gen = random.Random(12345)

# Seeding with no argument auto-seeds, producing a fresh state each time.
gen.seed()
auto1 = gen.getstate()
gen.seed()
auto2 = gen.getstate()
assert auto1 != auto2, "autoseed should differ"

print("no_arg_seed_autoseeds_fresh_state OK")
