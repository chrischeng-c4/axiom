# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "system_random_random_and_randint"
# subject = "random.SystemRandom"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.SystemRandom: SystemRandom.random() stays in [0, 1) and randint endpoints are inclusive (degenerate randint(5,5) == 5)"""
import random

gen = random.SystemRandom()
r = gen.random()
assert isinstance(r, float) and 0.0 <= r < 1.0, f"random = {r!r}"
assert gen.randint(5, 5) == 5, "degenerate randint"

print("system_random_random_and_randint OK")
