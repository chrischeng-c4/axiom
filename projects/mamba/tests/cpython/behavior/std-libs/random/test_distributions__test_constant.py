# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "test_distributions__test_constant"
# subject = "cpython.test_random.TestDistributions.test_constant"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_random.py::TestDistributions::test_constant
"""Auto-ported test: TestDistributions::test_constant (CPython 3.12 oracle)."""


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
g = random.Random()
N = 100
for variate, args, expected in [(g.uniform, (10.0, 10.0), 10.0), (g.triangular, (10.0, 10.0), 10.0), (g.triangular, (10.0, 10.0, 10.0), 10.0), (g.expovariate, (float('inf'),), 0.0), (g.vonmisesvariate, (3.0, float('inf')), 3.0), (g.gauss, (10.0, 0.0), 10.0), (g.lognormvariate, (0.0, 0.0), 1.0), (g.lognormvariate, (-float('inf'), 0.0), 0.0), (g.normalvariate, (10.0, 0.0), 10.0), (g.binomialvariate, (0, 0.5), 0), (g.binomialvariate, (10, 0.0), 0), (g.binomialvariate, (10, 1.0), 10), (g.paretovariate, (float('inf'),), 1.0), (g.weibullvariate, (10.0, float('inf')), 10.0), (g.weibullvariate, (0.0, 10.0), 0.0)]:
    for i in range(N):

        assert variate(*args) == expected
print("TestDistributions::test_constant: ok")
