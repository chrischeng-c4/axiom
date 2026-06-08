# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "binomialvariate_stays_in_range"
# subject = "random.Random.binomialvariate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.binomialvariate: binomialvariate(5, 0.25) stays within [0, 5] for ordinary parameters across 50 seeded draws"""
import random

gen = random.Random(0)
for _ in range(50):
    assert gen.binomialvariate(5, 0.25) in range(6), "binomial out of [0,5]"

print("binomialvariate_stays_in_range OK")
