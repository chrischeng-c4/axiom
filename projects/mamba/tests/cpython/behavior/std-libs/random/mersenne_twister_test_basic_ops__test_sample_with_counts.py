# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "mersenne_twister_test_basic_ops__test_sample_with_counts"
# subject = "cpython.test_random.MersenneTwister_TestBasicOps.test_sample_with_counts"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_random.py::MersenneTwister_TestBasicOps::test_sample_with_counts
"""Auto-ported test: MersenneTwister_TestBasicOps::test_sample_with_counts (CPython 3.12 oracle)."""


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
colors = ['red', 'green', 'blue', 'orange', 'black', 'brown', 'amber']
counts = [500, 200, 20, 10, 5, 0, 1]
k = 700
summary = Counter(sample(colors, counts=counts, k=k))

assert sum(summary.values()) == k
for color, weight in zip(colors, counts):

    assert summary[color] <= weight

assert 'brown' not in summary
k = sum(counts)
summary = Counter(sample(colors, counts=counts, k=k))

assert sum(summary.values()) == k
for color, weight in zip(colors, counts):

    assert summary[color] <= weight

assert 'brown' not in summary
summary = Counter(sample(['x'], counts=[10], k=8))

assert summary == Counter(x=8)
nc = len(colors)
summary = Counter(sample(colors, counts=[10] * nc, k=10 * nc))

assert summary == Counter(10 * colors)
try:
    sample(['red', 'green', 'blue'], counts=10, k=10)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    sample(['red', 'green', 'blue'], counts=[-3, -7, -8], k=2)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    sample(['red', 'green'], counts=[10, 10], k=21)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    sample(['red', 'green', 'blue'], counts=[1, 2], k=2)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    sample(['red', 'green', 'blue'], counts=[1, 2, 3, 4], k=2)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert sample('abc', k=0, counts=[0, 0, 0]) == sample([], k=0)

assert sample([], 0, counts=[]) == sample([], 0)
try:
    sample([], 1, counts=[])
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    sample('x', 1, counts=[0])
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("MersenneTwister_TestBasicOps::test_sample_with_counts: ok")
