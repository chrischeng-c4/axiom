# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "sample_nonsequence_population_raises_typeerror"
# subject = "random.Random.sample"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.sample: sample requires a sequence: dict and set populations are both rejected with TypeError"""
import random

gen = random.Random(0)

# A dict population is rejected with TypeError.
try:
    gen.sample(dict.fromkeys("abcdef"), 2)
    raise AssertionError("expected TypeError for dict population")
except TypeError:
    pass

# A set population is rejected with TypeError.
try:
    gen.sample({10, 20, 30, 40, 50}, k=2)
    raise AssertionError("expected TypeError for set population")
except TypeError:
    pass

print("sample_nonsequence_population_raises_typeerror OK")
