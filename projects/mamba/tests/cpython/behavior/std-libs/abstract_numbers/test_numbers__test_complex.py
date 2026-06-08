# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abstract_numbers"
# dimension = "behavior"
# case = "test_numbers__test_complex"
# subject = "cpython.test_abstract_numbers.TestNumbers.test_complex"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_abstract_numbers.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_abstract_numbers.py::TestNumbers::test_complex
"""Auto-ported test: TestNumbers::test_complex (CPython 3.12 oracle)."""


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

assert not issubclass(complex, Integral)

assert not issubclass(complex, Rational)

assert not issubclass(complex, Real)

assert issubclass(complex, Complex)

assert issubclass(complex, Number)
c1, c2 = (complex(3, 2), complex(4, 1))

try:
    math.trunc(c1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.mod(c1, c2)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    divmod(c1, c2)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.floordiv(c1, c2)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    float(c1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    int(c1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestNumbers::test_complex: ok")
