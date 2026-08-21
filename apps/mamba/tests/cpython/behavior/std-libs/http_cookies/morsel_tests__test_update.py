# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "morsel_tests__test_update"
# subject = "cpython.test_http_cookies.MorselTests.test_update"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::MorselTests::test_update
"""Auto-ported test: MorselTests::test_update (CPython 3.12 oracle)."""


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
attribs = {'expires': 1, 'Version': 2, 'DOMAIN': 'example.com'}
morsel = cookies.Morsel()
morsel.update(attribs)

assert morsel['expires'] == 1

assert morsel['version'] == 2

assert morsel['domain'] == 'example.com'
morsel = cookies.Morsel()
morsel.update(list(attribs.items()))

assert morsel['expires'] == 1

assert morsel['version'] == 2

assert morsel['domain'] == 'example.com'
morsel = cookies.Morsel()
morsel.update(((k, v) for k, v in attribs.items()))

assert morsel['expires'] == 1

assert morsel['version'] == 2

assert morsel['domain'] == 'example.com'
try:
    morsel.update({'invalid': 'value'})
    raise AssertionError('expected cookies.CookieError')
except cookies.CookieError:
    pass

assert 'invalid' not in morsel

try:
    morsel.update()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    morsel.update(0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("MorselTests::test_update: ok")
