# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "morsel_tests__test_defaults"
# subject = "cpython.test_http_cookies.MorselTests.test_defaults"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::MorselTests::test_defaults
"""Auto-ported test: MorselTests::test_defaults (CPython 3.12 oracle)."""


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

assert morsel.key is None

assert morsel.value is None

assert morsel.coded_value is None

assert morsel.keys() == cookies.Morsel._reserved.keys()
for key, val in morsel.items():

    assert val == ''
print("MorselTests::test_defaults: ok")
