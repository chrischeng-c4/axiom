# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "randrange_excludes_stop"
# subject = "random.randrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.randrange: randrange(5) yields values in the half-open range [0, 5): all draws satisfy 0 <= v < 5 and the top value 4 does appear"""
import random

random.seed(2)
rr = [random.randrange(5) for _ in range(50)]
assert all(0 <= v < 5 for v in rr), f"randrange [0,5): {rr!r}"
assert 4 in rr, "4 should appear"

print("randrange_excludes_stop OK")
