# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "morsel_tests__test_setdefault"
# subject = "cpython.test_http_cookies.MorselTests.test_setdefault"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::MorselTests::test_setdefault
"""Auto-ported test: MorselTests::test_setdefault (CPython 3.12 oracle)."""


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
morsel.update({'domain': 'example.com', 'version': 2})

assert morsel.setdefault('expires', 'value') == ''

assert morsel['expires'] == ''

assert morsel.setdefault('Version', 1) == 2

assert morsel['version'] == 2

assert morsel.setdefault('DOMAIN', 'value') == 'example.com'

assert morsel['domain'] == 'example.com'
try:
    morsel.setdefault('invalid', 'value')
    raise AssertionError('expected cookies.CookieError')
except cookies.CookieError:
    pass

assert 'invalid' not in morsel
print("MorselTests::test_setdefault: ok")
