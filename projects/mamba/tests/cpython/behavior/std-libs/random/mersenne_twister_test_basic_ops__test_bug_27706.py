# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "mersenne_twister_test_basic_ops__test_bug_27706"
# subject = "cpython.test_random.MersenneTwister_TestBasicOps.test_bug_27706"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_random.py::MersenneTwister_TestBasicOps::test_bug_27706
"""Auto-ported test: MersenneTwister_TestBasicOps::test_bug_27706 (CPython 3.12 oracle)."""


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
gen.seed('nofar', version=1)

assert [gen.random().hex() for i in range(4)] == ['0x1.8645314505ad7p-1', '0x1.afb1f82e40a40p-5', '0x1.2a59d2285e971p-1', '0x1.56977142a7880p-6']
gen.seed('rachel', version=1)

assert [gen.random().hex() for i in range(4)] == ['0x1.0b294cc856fcdp-1', '0x1.2ad22d79e77b8p-3', '0x1.3052b9c072678p-2', '0x1.578f332106574p-3']
gen.seed('', version=1)

assert [gen.random().hex() for i in range(4)] == ['0x1.b0580f98a7dbep-1', '0x1.84129978f9c1ap-1', '0x1.aeaa51052e978p-2', '0x1.092178fb945a6p-2']
print("MersenneTwister_TestBasicOps::test_bug_27706: ok")
