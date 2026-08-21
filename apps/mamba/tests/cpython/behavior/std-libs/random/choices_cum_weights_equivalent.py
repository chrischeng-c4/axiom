# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "choices_cum_weights_equivalent"
# subject = "random.Random.choices"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.choices: cum_weights is the prefix-sum form of weights and selects identically: choices('abcd', cum_weights=[1,1,1,1]) is ['a'] and cum_weights=[0,0,0,1] is ['d']"""
import random

gen = random.Random(0)
assert gen.choices("abcd", cum_weights=[1, 1, 1, 1]) == ["a"], "cum pick first"
assert gen.choices("abcd", cum_weights=[0, 0, 0, 1]) == ["d"], "cum pick last"

print("choices_cum_weights_equivalent OK")
