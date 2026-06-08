# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "sample_counts_respects_multiplicity_caps"
# subject = "random.Random.sample"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.sample: counts expands each element by its multiplicity and respects the caps: sampling k=700 from colors with counts gives sum 700, each color within its cap, and a zero-count element excluded"""
import random

from collections import Counter

gen = random.Random(0)
colors = ["red", "green", "blue", "orange", "black", "brown", "amber"]
counts = [500, 200, 20, 10, 5, 0, 1]
summary = Counter(gen.sample(colors, counts=counts, k=700))
assert sum(summary.values()) == 700, f"total = {sum(summary.values())!r}"
for color, cap in zip(colors, counts):
    assert summary[color] <= cap, f"{color}: {summary[color]} > {cap}"
assert "brown" not in summary, "zero-count element excluded"

print("sample_counts_respects_multiplicity_caps OK")
