# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "choice_covers_whole_sequence"
# subject = "random.choice"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.choice: choice draws only from the sequence and, over enough seeded draws, covers every element: choice(['a','b','c']) eventually yields all three"""
import random

random.seed(3)
items = ["a", "b", "c"]
choices = {random.choice(items) for _ in range(50)}
assert choices == {"a", "b", "c"}, f"choice covers all: {choices!r}"

print("choice_covers_whole_sequence OK")
