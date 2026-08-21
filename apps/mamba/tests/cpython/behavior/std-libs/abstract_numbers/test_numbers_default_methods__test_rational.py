# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abstract_numbers"
# dimension = "behavior"
# case = "test_numbers_default_methods__test_rational"
# subject = "cpython.test_abstract_numbers.TestNumbersDefaultMethods.test_rational"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_abstract_numbers.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_abstract_numbers.py::TestNumbersDefaultMethods::test_rational
"""Auto-ported test: TestNumbersDefaultMethods::test_rational (CPython 3.12 oracle)."""


import abc
import math
import operator
import unittest
from numbers import Complex, Real, Rational, Integral, Number


'Unit tests for numbers.py.'

def concretize(cls):

    def not_implemented(*args, **kwargs):
        raise NotImplementedError()
    for name in dir(cls):
        try:
            value = getattr(cls, name)
            if value.__isabstractmethod__:
                setattr(cls, name, not_implemented)
        except AttributeError:
            pass
    abc.update_abstractmethods(cls)
    return cls


# --- test body ---
@concretize
class MyRational(Rational):

    def __init__(self, numerator, denominator):
        self.n = numerator
        self.d = denominator

    @property
    def numerator(self):
        return self.n

    @property
    def denominator(self):
        return self.d

assert float(MyRational(5, 2)) == 2.5
print("TestNumbersDefaultMethods::test_rational: ok")
