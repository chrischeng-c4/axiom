# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "sample_negative_counts_raises_valueerror"
# subject = "random.Random.sample"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.sample: negative counts raise ValueError, and k larger than the expanded population also raises ValueError"""
import random

gen = random.Random(0)

# Negative counts raise ValueError.
try:
    gen.sample(["red", "green", "blue"], counts=[-3, -7, -8], k=2)
    raise AssertionError("expected ValueError for negative counts")
except ValueError:
    pass

# k larger than the expanded population raises ValueError.
try:
    gen.sample(["red", "green"], counts=[10, 10], k=21)
    raise AssertionError("expected ValueError for k > total")
except ValueError:
    pass

print("sample_negative_counts_raises_valueerror OK")
