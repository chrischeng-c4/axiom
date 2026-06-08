# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "sample_has_no_duplicates"
# subject = "random.sample"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.sample: sample draws without replacement: sample(range(20), 10) returns 10 distinct elements, all drawn from the pool"""
import random

random.seed(4)
pool = list(range(20))
s = random.sample(pool, 10)
assert len(s) == 10, f"sample len = {len(s)!r}"
assert len(set(s)) == 10, "sample no duplicates"
assert all(v in pool for v in s), "sample from pool"

print("sample_has_no_duplicates OK")
