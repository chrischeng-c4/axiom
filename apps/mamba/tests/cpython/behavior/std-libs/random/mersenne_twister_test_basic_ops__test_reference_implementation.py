# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "mersenne_twister_test_basic_ops__test_reference_implementation"
# subject = "cpython.test_random.MersenneTwister_TestBasicOps.test_referenceImplementation"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_random.py::MersenneTwister_TestBasicOps::test_referenceImplementation
"""Auto-ported test: MersenneTwister_TestBasicOps::test_referenceImplementation (CPython 3.12 oracle)."""


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

def randomlist(n):
    """Helper function to make a list of random numbers"""
    return [gen.random() for i in range(n)]
expected = [0.4583980307371326, 0.8605781520197878, 0.9284833172678215, 0.3593268111978246, 0.08182349376244957, 0.1433222647016933, 0.08429782382352002, 0.5381486467183145, 0.0892150249119934, 0.7848619610537291]
gen.seed(61731 + (24903 << 32) + (614 << 64) + (42143 << 96))
actual = randomlist(2000)[-10:]
for a, e in zip(actual, expected):

    assert abs(a - e) < 1e-07
print("MersenneTwister_TestBasicOps::test_referenceImplementation: ok")
