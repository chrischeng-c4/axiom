# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "cookie_tests__test_unquote_large"
# subject = "cpython.test_http_cookies.CookieTests.test_unquote_large"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::CookieTests::test_unquote_large
"""Auto-ported test: CookieTests::test_unquote_large (CPython 3.12 oracle)."""


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
n = 10 ** 6
for encoded in ('\\\\', '\\134'):
    data = 'a="b=' + encoded * n + ';"'
    C = cookies.SimpleCookie()
    C.load(data)
    value = C['a'].value

    assert value[:3] == 'b=\\'

    assert value[-2:] == '\\;'

    assert len(value) == n + 3
print("CookieTests::test_unquote_large: ok")
