# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "mersenne_twister_test_basic_ops__test_choices_algorithms"
# subject = "cpython.test_random.MersenneTwister_TestBasicOps.test_choices_algorithms"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::MersenneTwister_TestBasicOps::test_choices_algorithms
"""Auto-ported test: MersenneTwister_TestBasicOps::test_choices_algorithms (CPython 3.12 oracle)."""


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
choices = gen.choices
n = 104729
gen.seed(8675309)
a = gen.choices(range(n), k=10000)
gen.seed(8675309)
b = gen.choices(range(n), [1] * n, k=10000)

assert a == b
gen.seed(8675309)
c = gen.choices(range(n), cum_weights=range(1, n + 1), k=10000)

assert a == c
population = ['Red', 'Black', 'Green']
weights = [18, 18, 2]
cum_weights = [18, 36, 38]
expanded_population = ['Red'] * 18 + ['Black'] * 18 + ['Green'] * 2
gen.seed(9035768)
a = gen.choices(expanded_population, k=10000)
gen.seed(9035768)
b = gen.choices(population, weights, k=10000)

assert a == b
gen.seed(9035768)
c = gen.choices(population, cum_weights=cum_weights, k=10000)

assert a == c
print("MersenneTwister_TestBasicOps::test_choices_algorithms: ok")
