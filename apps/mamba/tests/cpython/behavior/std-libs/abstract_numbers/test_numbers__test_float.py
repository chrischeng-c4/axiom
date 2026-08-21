# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abstract_numbers"
# dimension = "behavior"
# case = "test_numbers__test_float"
# subject = "cpython.test_abstract_numbers.TestNumbers.test_float"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_abstract_numbers.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_abstract_numbers.py::TestNumbers::test_float
"""Auto-ported test: TestNumbers::test_float (CPython 3.12 oracle)."""


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

assert not issubclass(float, Integral)

assert not issubclass(float, Rational)

assert issubclass(float, Real)

assert issubclass(float, Complex)

assert issubclass(float, Number)

assert 7.3 == float(7.3).real

assert 0 == float(7.3).imag

assert 7.3 == float(7.3).conjugate()

assert -7.3 == float(-7.3).conjugate()
print("TestNumbers::test_float: ok")
