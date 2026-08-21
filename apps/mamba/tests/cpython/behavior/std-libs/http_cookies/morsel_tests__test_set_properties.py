# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "morsel_tests__test_set_properties"
# subject = "cpython.test_http_cookies.MorselTests.test_set_properties"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::MorselTests::test_set_properties
"""Auto-ported test: MorselTests::test_set_properties (CPython 3.12 oracle)."""


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
try:
    morsel.key = ''
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
try:
    morsel.value = ''
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
try:
    morsel.coded_value = ''
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
print("MorselTests::test_set_properties: ok")
