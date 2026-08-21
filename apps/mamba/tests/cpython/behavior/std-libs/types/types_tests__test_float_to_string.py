# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "types_tests__test_float_to_string"
# subject = "cpython.test_types.TypesTests.test_float_to_string"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import collections.abc
from collections import namedtuple
import copy
import gc
import inspect
import pickle
import locale
import sys
import textwrap
import types
import weakref
import typing

def test(f, result):
    assert f.__format__('e') == result
    assert '%e' % f == result
for i in range(-99, 100):
    test(float('1.5e' + str(i)), '1.500000e{0:+03d}'.format(i))
assert 1.5e+100.__format__('e') == '1.500000e+100'
assert '%e' % 1.5e+100 == '1.500000e+100'
assert 1.5e+101.__format__('e') == '1.500000e+101'
assert '%e' % 1.5e+101 == '1.500000e+101'
assert 1.5e-100.__format__('e') == '1.500000e-100'
assert '%e' % 1.5e-100 == '1.500000e-100'
assert 1.5e-101.__format__('e') == '1.500000e-101'
assert '%e' % 1.5e-101 == '1.500000e-101'
assert '%g' % 1.0 == '1'
assert '%#g' % 1.0 == '1.00000'

print("TypesTests::test_float_to_string: ok")
