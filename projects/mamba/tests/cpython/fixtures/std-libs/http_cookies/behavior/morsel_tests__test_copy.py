# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "morsel_tests__test_copy"
# subject = "cpython.test_http_cookies.MorselTests.test_copy"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::MorselTests::test_copy
"""Auto-ported test: MorselTests::test_copy (CPython 3.12 oracle)."""


import copy
import unittest
import doctest
from http import cookies
import pickle
from test import support


def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(cookies))
    return tests


# --- test body ---
morsel_a = cookies.Morsel()
morsel_a.set('foo', 'bar', 'baz')
morsel_a.update({'version': 2, 'comment': 'foo'})
morsel_b = morsel_a.copy()

assert isinstance(morsel_b, cookies.Morsel)

assert morsel_a is not morsel_b

assert morsel_a == morsel_b
morsel_b = copy.copy(morsel_a)

assert isinstance(morsel_b, cookies.Morsel)

assert morsel_a is not morsel_b

assert morsel_a == morsel_b
print("MorselTests::test_copy: ok")
