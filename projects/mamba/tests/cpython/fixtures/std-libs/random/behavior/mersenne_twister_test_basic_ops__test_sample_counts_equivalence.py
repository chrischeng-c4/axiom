# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "mersenne_twister_test_basic_ops__test_sample_counts_equivalence"
# subject = "cpython.test_random.MersenneTwister_TestBasicOps.test_sample_counts_equivalence"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_random.py::MersenneTwister_TestBasicOps::test_sample_counts_equivalence
"""Auto-ported test: MersenneTwister_TestBasicOps::test_sample_counts_equivalence (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
gen = random.Random()
sample = gen.sample
seed = gen.seed
colors = ['red', 'green', 'blue', 'orange', 'black', 'amber']
counts = [500, 200, 20, 10, 5, 1]
k = 700
seed(8675309)
s1 = sample(colors, counts=counts, k=k)
seed(8675309)
expanded = [color for color, count in zip(colors, counts) for i in range(count)]

assert len(expanded) == sum(counts)
s2 = sample(expanded, k=k)

assert s1 == s2
pop = 'abcdefghi'
counts = [10, 9, 8, 7, 6, 5, 4, 3, 2]
seed(8675309)
s1 = ''.join(sample(pop, counts=counts, k=30))
expanded = ''.join([letter for letter, count in zip(pop, counts) for i in range(count)])
seed(8675309)
s2 = ''.join(sample(expanded, k=30))

assert s1 == s2
print("MersenneTwister_TestBasicOps::test_sample_counts_equivalence: ok")
