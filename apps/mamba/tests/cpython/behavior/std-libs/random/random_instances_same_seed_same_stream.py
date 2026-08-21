# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "random_instances_same_seed_same_stream"
# subject = "random.Random"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random: two independent Random(10) instances are deterministic and identical: each produces the same [randint(0,100) for _ in range(5)] sequence"""
import random

rng1 = random.Random(10)
rng2 = random.Random(10)
from1 = [rng1.randint(0, 100) for _ in range(5)]
from2 = [rng2.randint(0, 100) for _ in range(5)]
assert from1 == from2, f"same seed = same seq: {from1!r} != {from2!r}"

print("random_instances_same_seed_same_stream OK")
