# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "test_distributions__test_zeroinputs"
# subject = "cpython.test_random.TestDistributions.test_zeroinputs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::TestDistributions::test_zeroinputs
"""Auto-ported test: TestDistributions::test_zeroinputs (CPython 3.12 oracle)."""


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
x = [g.random() for i in range(50)] + [0.0] * 5
g.random = x[:].pop
g.uniform(1, 10)
g.random = x[:].pop
g.paretovariate(1.0)
g.random = x[:].pop
g.expovariate(1.0)
g.random = x[:].pop
g.expovariate()
g.random = x[:].pop
g.weibullvariate(1.0, 1.0)
g.random = x[:].pop
g.vonmisesvariate(1.0, 1.0)
g.random = x[:].pop
g.normalvariate(0.0, 1.0)
g.random = x[:].pop
g.gauss(0.0, 1.0)
g.random = x[:].pop
g.lognormvariate(0.0, 1.0)
g.random = x[:].pop
g.vonmisesvariate(0.0, 1.0)
g.random = x[:].pop
g.gammavariate(0.01, 1.0)
g.random = x[:].pop
g.gammavariate(1.0, 1.0)
g.random = x[:].pop
g.gammavariate(200.0, 1.0)
g.random = x[:].pop
g.betavariate(3.0, 3.0)
g.random = x[:].pop
g.triangular(0.0, 1.0, 1.0 / 3.0)
print("TestDistributions::test_zeroinputs: ok")
