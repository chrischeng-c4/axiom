# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "choices_nonpositive_k_returns_empty"
# subject = "random.Random.choices"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.choices: k == 0 and k < 0 both yield an empty list: choices('abcd', k=0) and choices('abcd', k=-1) are both []"""
import random

gen = random.Random(0)
assert gen.choices("abcd", k=0) == [], "k=0 empty"
assert gen.choices("abcd", k=-1) == [], "k<0 empty"

print("choices_nonpositive_k_returns_empty OK")
