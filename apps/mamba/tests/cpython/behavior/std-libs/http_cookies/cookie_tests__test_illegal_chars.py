# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "cookie_tests__test_illegal_chars"
# subject = "cpython.test_http_cookies.CookieTests.test_illegal_chars"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::CookieTests::test_illegal_chars
"""Auto-ported test: CookieTests::test_illegal_chars (CPython 3.12 oracle)."""


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
rawdata = 'a=b; c,d=e'
C = cookies.SimpleCookie()
try:
    C.load(rawdata)
    raise AssertionError('expected cookies.CookieError')
except cookies.CookieError:
    pass
print("CookieTests::test_illegal_chars: ok")
