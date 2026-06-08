# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "uniform_stays_within_bounds"
# subject = "random.uniform"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.uniform: uniform(2.0, 5.0) returns floats within the closed bound [2.0, 5.0] across 100 seeded draws"""
import random

random.seed(6)
us = [random.uniform(2.0, 5.0) for _ in range(100)]
assert all(2.0 <= u <= 5.0 for u in us), "uniform in [2,5]"

print("uniform_stays_within_bounds OK")
