# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "choices_single_weight_is_deterministic"
# subject = "random.Random.choices"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.choices: a weight vector targeting one element forces that draw: choices('abcd', [1,0,0,0]) is ['a'] and choices('abcd', [0,0,0,1]) is ['d']"""
import random

gen = random.Random(0)
assert gen.choices("abcd", [1, 0, 0, 0]) == ["a"], "weights pick first"
assert gen.choices("abcd", [0, 0, 0, 1]) == ["d"], "weights pick last"

print("choices_single_weight_is_deterministic OK")
