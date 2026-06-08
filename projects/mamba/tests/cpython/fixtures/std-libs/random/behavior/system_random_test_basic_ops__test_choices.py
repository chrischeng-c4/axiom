# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "system_random_test_basic_ops__test_choices"
# subject = "cpython.test_random.SystemRandom_TestBasicOps.test_choices"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_random.py::SystemRandom_TestBasicOps::test_choices
"""Auto-ported test: SystemRandom_TestBasicOps::test_choices (CPython 3.12 oracle)."""


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
gen = random.SystemRandom()
choices = gen.choices
data = ['red', 'green', 'blue', 'yellow']
str_data = 'abcd'
range_data = range(4)
set_data = set(range(4))
for sample in [choices(data, k=5), choices(data, range(4), k=5), choices(k=5, population=data, weights=range(4)), choices(k=5, population=data, cum_weights=range(4))]:

    assert len(sample) == 5

    assert type(sample) == list

    assert set(sample) <= set(data)
try:
    choices(2)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert choices(data, k=0) == []

assert choices(data, k=-1) == []
try:
    choices(data, k=2.5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert set(choices(str_data, k=5)) <= set(str_data)

assert set(choices(range_data, k=5)) <= set(range_data)
try:
    choices(set_data, k=2)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert set(choices(data, None, k=5)) <= set(data)

assert set(choices(data, weights=None, k=5)) <= set(data)
try:
    choices(data, [1, 2], k=5)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    choices(data, 10, k=5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    choices(data, [None] * 4, k=5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
for weights in [[15, 10, 25, 30], [15.1, 10.2, 25.2, 30.3], [Fraction(1, 3), Fraction(2, 6), Fraction(3, 6), Fraction(4, 6)], [True, False, True, False]]:

    assert set(choices(data, weights, k=5)) <= set(data)
try:
    choices(data, cum_weights=[1, 2], k=5)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    choices(data, cum_weights=10, k=5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    choices(data, cum_weights=[None] * 4, k=5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    choices(data, range(4), cum_weights=range(4), k=5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
for weights in [[15, 10, 25, 30], [15.1, 10.2, 25.2, 30.3], [Fraction(1, 3), Fraction(2, 6), Fraction(3, 6), Fraction(4, 6)]]:

    assert set(choices(data, cum_weights=weights, k=5)) <= set(data)

assert choices('abcd', [1, 0, 0, 0]) == ['a']

assert choices('abcd', [0, 1, 0, 0]) == ['b']

assert choices('abcd', [0, 0, 1, 0]) == ['c']

assert choices('abcd', [0, 0, 0, 1]) == ['d']
try:
    choices([], k=1)
    raise AssertionError('expected IndexError')
except IndexError:
    pass
try:
    choices([], weights=[], k=1)
    raise AssertionError('expected IndexError')
except IndexError:
    pass
try:
    choices([], cum_weights=[], k=5)
    raise AssertionError('expected IndexError')
except IndexError:
    pass
print("SystemRandom_TestBasicOps::test_choices: ok")
