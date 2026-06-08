# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "test_distributions__test_binomialvariate"
# subject = "cpython.test_random.TestDistributions.test_binomialvariate"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_random.py::TestDistributions::test_binomialvariate
"""Auto-ported test: TestDistributions::test_binomialvariate (CPython 3.12 oracle)."""


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
B = random.binomialvariate
try:
    B(n=-1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    B(n=1, p=-0.5)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    B(n=1, p=1.5)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert B(10, 0.0) == 0

assert B(10, 1.0) == 10

assert B(1, 0.3) in {0, 1}

assert B(1, 0.9) in {0, 1}

assert B(1, 0.0) in {0}

assert B(1, 1.0) in {1}

assert B(5, 0.25) in set(range(6))

assert B(5, 0.75) in set(range(6))

assert B(100, 0.25) in set(range(101))

assert B(100, 0.75) in set(range(101))
c = Counter((B(4, 0.25) for i in range(100000)))

assert 29641 <= c[0] <= 33641

assert 40188 <= c[1] <= 44188

assert 19094 <= c[2] <= 23094

assert 2688 <= c[3] <= 6688

assert set(c) == {0, 1, 2, 3, 4}
c = Counter((B(100, 0.25) for i in range(100000)))

assert 34214 <= c[20] + c[21] + c[22] + c[23] + c[24] <= 38214

assert set(c) <= set(range(101))

assert c.total() == 100000

assert 19000000 <= B(100000000, 0.2) <= 21000000

assert 89000000 <= B(100000000, 0.9) <= 91000000
print("TestDistributions::test_binomialvariate: ok")
