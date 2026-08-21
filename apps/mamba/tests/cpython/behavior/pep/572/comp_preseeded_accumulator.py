# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "comp_preseeded_accumulator"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a pre-seeded accumulator outside the comprehension is honored by the walrus accumulation inside it"""
# Pre-seeding the accumulator outside the comprehension is honored.
acc = 0
sums = [(acc := i + acc) for i in range(5)]
assert sums == [0, 1, 3, 6, 10]
assert acc == 10

print("comp_preseeded_accumulator OK")
