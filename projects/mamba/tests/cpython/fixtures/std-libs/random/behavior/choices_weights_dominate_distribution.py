# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "choices_weights_dominate_distribution"
# subject = "random.choices"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.choices: weights bias the distribution: choices([0,1], weights=[1,99], k=100) yields ones for the large majority (count of 1 exceeds 80)"""
import random

random.seed(7)
weighted = random.choices([0, 1], weights=[1, 99], k=100)
# With weight 99x, 1 should dominate.
assert weighted.count(1) > 80, f"weighted choices: {weighted.count(1)}/100 ones"

print("choices_weights_dominate_distribution OK")
