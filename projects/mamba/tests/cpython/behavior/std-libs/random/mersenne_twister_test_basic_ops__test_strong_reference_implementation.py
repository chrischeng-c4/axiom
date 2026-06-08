# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "mersenne_twister_test_basic_ops__test_strong_reference_implementation"
# subject = "cpython.test_random.MersenneTwister_TestBasicOps.test_strong_reference_implementation"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_random.py::MersenneTwister_TestBasicOps::test_strong_reference_implementation
"""Auto-ported test: MersenneTwister_TestBasicOps::test_strong_reference_implementation (CPython 3.12 oracle)."""


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
from math import ldexp
expected = [4128882400830239, 7751398889519013, 8363034243334166, 3236528186029503, 737000512037440, 1290932195808883, 759287295919497, 4847212089661076, 803577505899006, 7069408070677702]
gen.seed(61731 + (24903 << 32) + (614 << 64) + (42143 << 96))
actual = randomlist(2000)[-10:]
for a, e in zip(actual, expected):

    assert int(ldexp(a, 53)) == e
print("MersenneTwister_TestBasicOps::test_strong_reference_implementation: ok")
