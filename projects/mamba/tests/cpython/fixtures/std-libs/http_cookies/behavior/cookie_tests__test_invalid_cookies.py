# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "cookie_tests__test_invalid_cookies"
# subject = "cpython.test_http_cookies.CookieTests.test_invalid_cookies"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::CookieTests::test_invalid_cookies
"""Auto-ported test: CookieTests::test_invalid_cookies (CPython 3.12 oracle)."""


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
C = cookies.SimpleCookie()
for s in (']foo=x', '[foo=x', 'blah]foo=x', 'blah[foo=x', 'Set-Cookie: foo=bar', 'Set-Cookie: foo', 'foo=bar; baz', 'baz; foo=bar', 'secure;foo=bar', 'Version=1;foo=bar'):
    C.load(s)

    assert dict(C) == {}

    assert C.output() == ''
print("CookieTests::test_invalid_cookies: ok")
