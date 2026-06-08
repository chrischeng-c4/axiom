# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "morsel_tests__test_eq"
# subject = "cpython.test_http_cookies.MorselTests.test_eq"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::MorselTests::test_eq
"""Auto-ported test: MorselTests::test_eq (CPython 3.12 oracle)."""


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
base_case = ('key', 'value', '"value"')
attribs = {'path': '/', 'comment': 'foo', 'domain': 'example.com', 'version': 2}
morsel_a = cookies.Morsel()
morsel_a.update(attribs)
morsel_a.set(*base_case)
morsel_b = cookies.Morsel()
morsel_b.update(attribs)
morsel_b.set(*base_case)

assert morsel_a == morsel_b

assert not morsel_a != morsel_b
cases = (('key', 'value', 'mismatch'), ('key', 'mismatch', '"value"'), ('mismatch', 'value', '"value"'))
for case_b in cases:
    morsel_b = cookies.Morsel()
    morsel_b.update(attribs)
    morsel_b.set(*case_b)

    assert not morsel_a == morsel_b

    assert morsel_a != morsel_b
morsel_b = cookies.Morsel()
morsel_b.update(attribs)
morsel_b.set(*base_case)
morsel_b['comment'] = 'bar'

assert not morsel_a == morsel_b

assert morsel_a != morsel_b

assert not cookies.Morsel() == 1

assert cookies.Morsel() != 1

assert not cookies.Morsel() == ''

assert cookies.Morsel() != ''
items = list(cookies.Morsel().items())

assert not cookies.Morsel() == items

assert cookies.Morsel() != items
morsel = cookies.Morsel()
morsel.set(*base_case)
morsel.update(attribs)

assert morsel == dict(morsel)

assert not morsel != dict(morsel)
print("MorselTests::test_eq: ok")
