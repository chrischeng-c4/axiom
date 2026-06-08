# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "test_random_subclassing__test_subclasses_overriding_methods"
# subject = "cpython.test_random.TestRandomSubclassing.test_subclasses_overriding_methods"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_random.py::TestRandomSubclassing::test_subclasses_overriding_methods
"""Auto-ported test: TestRandomSubclassing::test_subclasses_overriding_methods (CPython 3.12 oracle)."""


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
class SubClass1(random.Random):

    def random(self):
        called.add('SubClass1.random')
        return random.Random.random(self)

    def getrandbits(self, n):
        called.add('SubClass1.getrandbits')
        return random.Random.getrandbits(self, n)
called = set()
SubClass1().randrange(42)

assert called == {'SubClass1.getrandbits'}

class SubClass2(random.Random):

    def random(self):
        called.add('SubClass2.random')
        return random.Random.random(self)
called = set()
SubClass2().randrange(42)

assert called == {'SubClass2.random'}

class SubClass3(SubClass2):

    def getrandbits(self, n):
        called.add('SubClass3.getrandbits')
        return random.Random.getrandbits(self, n)
called = set()
SubClass3().randrange(42)

assert called == {'SubClass3.getrandbits'}

class SubClass4(SubClass3):

    def random(self):
        called.add('SubClass4.random')
        return random.Random.random(self)
called = set()
SubClass4().randrange(42)

assert called == {'SubClass4.random'}

class Mixin1:

    def random(self):
        called.add('Mixin1.random')
        return random.Random.random(self)

class Mixin2:

    def getrandbits(self, n):
        called.add('Mixin2.getrandbits')
        return random.Random.getrandbits(self, n)

class SubClass5(Mixin1, random.Random):
    pass
called = set()
SubClass5().randrange(42)

assert called == {'Mixin1.random'}

class SubClass6(Mixin2, random.Random):
    pass
called = set()
SubClass6().randrange(42)

assert called == {'Mixin2.getrandbits'}

class SubClass7(Mixin1, Mixin2, random.Random):
    pass
called = set()
SubClass7().randrange(42)

assert called == {'Mixin1.random'}

class SubClass8(Mixin2, Mixin1, random.Random):
    pass
called = set()
SubClass8().randrange(42)

assert called == {'Mixin2.getrandbits'}
print("TestRandomSubclassing::test_subclasses_overriding_methods: ok")
