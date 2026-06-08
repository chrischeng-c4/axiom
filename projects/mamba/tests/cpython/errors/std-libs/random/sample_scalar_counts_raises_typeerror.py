# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "sample_scalar_counts_raises_typeerror"
# subject = "random.Random.sample"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.sample: a scalar counts argument raises TypeError: sample(['red','green','blue'], counts=10, k=2) raises"""
import random

gen = random.Random(0)
try:
    gen.sample(["red", "green", "blue"], counts=10, k=2)
    raise AssertionError("expected TypeError for scalar counts")
except TypeError:
    pass

print("sample_scalar_counts_raises_typeerror OK")
