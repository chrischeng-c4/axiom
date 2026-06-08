# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abstract_numbers"
# dimension = "behavior"
# case = "test_numbers_default_methods__test_complex"
# subject = "cpython.test_abstract_numbers.TestNumbersDefaultMethods.test_complex"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_abstract_numbers.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_abstract_numbers.py::TestNumbersDefaultMethods::test_complex
"""Auto-ported test: TestNumbersDefaultMethods::test_complex (CPython 3.12 oracle)."""


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
class MyComplex(Complex):

    def __init__(self, real, imag):
        self.r = real
        self.i = imag

    @property
    def real(self):
        return self.r

    @property
    def imag(self):
        return self.i

    def __add__(self, other):
        if isinstance(other, Complex):
            return MyComplex(self.imag + other.imag, self.real + other.real)
        raise NotImplementedError

    def __neg__(self):
        return MyComplex(-self.real, -self.imag)

    def __eq__(self, other):
        if isinstance(other, Complex):
            return self.imag == other.imag and self.real == other.real
        if isinstance(other, Number):
            return self.imag == 0 and self.real == other.real

assert bool(MyComplex(1, 1))

assert bool(MyComplex(0, 1))

assert bool(MyComplex(1, 0))

assert not bool(MyComplex(0, 0))

assert MyComplex(2, 3) - complex(1, 2) == MyComplex(1, 1)

assert complex(2, 3) - MyComplex(1, 2) == MyComplex(1, 1)
print("TestNumbersDefaultMethods::test_complex: ok")
