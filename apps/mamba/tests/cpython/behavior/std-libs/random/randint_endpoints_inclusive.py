# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "randint_endpoints_inclusive"
# subject = "random.randint"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.randint: randint endpoints are inclusive: seeded random.randint(1, 2) over 100 draws produces both 1 and 2"""
import random

random.seed(1)
vals = {random.randint(1, 2) for _ in range(100)}
assert 1 in vals and 2 in vals, f"both endpoints seen: {vals!r}"

print("randint_endpoints_inclusive OK")
