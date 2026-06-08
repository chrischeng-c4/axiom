# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "morsel_tests__test_pickle"
# subject = "cpython.test_http_cookies.MorselTests.test_pickle"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::MorselTests::test_pickle
"""Auto-ported test: MorselTests::test_pickle (CPython 3.12 oracle)."""


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
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    morsel_b = pickle.loads(pickle.dumps(morsel_a, proto))

    assert isinstance(morsel_b, cookies.Morsel)

    assert morsel_b == morsel_a

    assert str(morsel_b) == str(morsel_a)
print("MorselTests::test_pickle: ok")
