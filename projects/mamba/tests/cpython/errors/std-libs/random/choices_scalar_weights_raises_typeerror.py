# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "choices_scalar_weights_raises_typeerror"
# subject = "random.Random.choices"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.choices: a scalar instead of a weight sequence raises TypeError: choices('abcd', 10, k=5) raises, as does supplying both weights and cum_weights"""
import random

gen = random.Random(0)

# A scalar instead of a weight sequence raises TypeError.
try:
    gen.choices("abcd", 10, k=5)
    raise AssertionError("expected TypeError for scalar weights")
except TypeError:
    pass

# Supplying both weights and cum_weights raises TypeError.
try:
    gen.choices("abcd", weights=range(4), cum_weights=range(4), k=5)
    raise AssertionError("expected TypeError for both weight kinds")
except TypeError:
    pass

print("choices_scalar_weights_raises_typeerror OK")
