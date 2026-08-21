# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "morsel_tests__test_setitem"
# subject = "cpython.test_http_cookies.MorselTests.test_setitem"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::MorselTests::test_setitem
"""Auto-ported test: MorselTests::test_setitem (CPython 3.12 oracle)."""


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
morsel = cookies.Morsel()
morsel['expires'] = 0

assert morsel['expires'] == 0
morsel['Version'] = 2

assert morsel['version'] == 2
morsel['DOMAIN'] = 'example.com'

assert morsel['domain'] == 'example.com'
try:
    morsel['invalid'] = 'value'
    raise AssertionError('expected cookies.CookieError')
except cookies.CookieError:
    pass

assert 'invalid' not in morsel
print("MorselTests::test_setitem: ok")
