# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "choices_uniform_draws_from_population"
# subject = "random.Random.choices"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.choices: uniform choices (no weights) return a list of length k drawn only from the population: choices('abcd', k=5) is a 5-element list whose elements are all in 'abcd'"""
import random

gen = random.Random(0)
out = gen.choices("abcd", k=5)
assert len(out) == 5, f"len = {len(out)!r}"
assert type(out) is list, f"type = {type(out)!r}"
assert set(out) <= set("abcd"), f"out = {out!r}"

print("choices_uniform_draws_from_population OK")
