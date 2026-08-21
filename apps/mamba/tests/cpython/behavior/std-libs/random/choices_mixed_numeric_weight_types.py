# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "choices_mixed_numeric_weight_types"
# subject = "random.Random.choices"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.choices: int, float, and bool weight vectors are all accepted: choices('abcd', weights, k=5) draws only from 'abcd' for each numeric weight kind"""
import random

gen = random.Random(0)
for weights in ([15, 10, 25, 30], [15.1, 10.2, 25.2, 30.3], [True, False, True, False]):
    sample = gen.choices("abcd", weights, k=5)
    assert set(sample) <= set("abcd"), f"mixed weights {weights!r} -> {sample!r}"

print("choices_mixed_numeric_weight_types OK")
