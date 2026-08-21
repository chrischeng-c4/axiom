# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abstract_numbers"
# dimension = "behavior"
# case = "test_numbers__test_int"
# subject = "cpython.test_abstract_numbers.TestNumbers.test_int"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_abstract_numbers.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_abstract_numbers.py::TestNumbers::test_int
"""Auto-ported test: TestNumbers::test_int (CPython 3.12 oracle)."""


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

assert issubclass(int, Integral)

assert issubclass(int, Rational)

assert issubclass(int, Real)

assert issubclass(int, Complex)

assert issubclass(int, Number)

assert 7 == int(7).real

assert 0 == int(7).imag

assert 7 == int(7).conjugate()

assert -7 == int(-7).conjugate()

assert 7 == int(7).numerator

assert 1 == int(7).denominator
print("TestNumbers::test_int: ok")
