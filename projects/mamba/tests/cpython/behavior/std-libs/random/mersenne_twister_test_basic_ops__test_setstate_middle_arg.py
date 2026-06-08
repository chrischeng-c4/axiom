# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "mersenne_twister_test_basic_ops__test_setstate_middle_arg"
# subject = "cpython.test_random.MersenneTwister_TestBasicOps.test_setstate_middle_arg"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_random.py::MersenneTwister_TestBasicOps::test_setstate_middle_arg
"""Auto-ported test: MersenneTwister_TestBasicOps::test_setstate_middle_arg (CPython 3.12 oracle)."""


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
start_state = gen.getstate()

try:
    gen.setstate((2, None, None))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    gen.setstate((2, (1, 2, 3), None))
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    gen.setstate((2, ('a',) * 625, None))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    gen.setstate((2, (0,) * 624 + ('a',), None))
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    gen.setstate((2, (1,) * 624 + (625,), None))
    raise AssertionError('expected (ValueError, OverflowError)')
except (ValueError, OverflowError):
    pass
try:
    gen.setstate((2, (1,) * 624 + (-1,), None))
    raise AssertionError('expected (ValueError, OverflowError)')
except (ValueError, OverflowError):
    pass
bits100 = gen.getrandbits(100)
gen.setstate(start_state)

assert gen.getrandbits(100) == bits100
state_values = gen.getstate()[1]
state_values = list(state_values)
state_values[-1] = float('nan')
state = (int(x) for x in state_values)

try:
    gen.setstate((2, state, None))
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("MersenneTwister_TestBasicOps::test_setstate_middle_arg: ok")
