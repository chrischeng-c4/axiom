# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "sample_counts_full_expanded_population"
# subject = "random.Random.sample"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.sample: drawing the entire expanded population returns every copy: sample(['x'], counts=[10], k=8) yields exactly 8 'x' values"""
import random

from collections import Counter

gen = random.Random(0)
got = Counter(gen.sample(["x"], counts=[10], k=8))
assert got == Counter(x=8), f"counts expansion = {got!r}"

print("sample_counts_full_expanded_population OK")
