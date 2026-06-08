# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abstract_numbers"
# dimension = "behavior"
# case = "test_numbers_default_methods__test_real"
# subject = "cpython.test_abstract_numbers.TestNumbersDefaultMethods.test_real"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_abstract_numbers.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_abstract_numbers.py::TestNumbersDefaultMethods::test_real
"""Auto-ported test: TestNumbersDefaultMethods::test_real (CPython 3.12 oracle)."""


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
class MyReal(Real):

    def __init__(self, n):
        self.n = n

    def __pos__(self):
        return self.n

    def __float__(self):
        return float(self.n)

    def __floordiv__(self, other):
        return self.n // other

    def __rfloordiv__(self, other):
        return other // self.n

    def __mod__(self, other):
        return self.n % other

    def __rmod__(self, other):
        return other % self.n

assert divmod(MyReal(3), 2) == (1, 1)

assert divmod(3, MyReal(2)) == (1, 1)

assert complex(MyReal(1)) == 1 + 0j

assert MyReal(3).real == 3

assert MyReal(3).imag == 0

assert MyReal(123).conjugate() == 123
print("TestNumbersDefaultMethods::test_real: ok")
