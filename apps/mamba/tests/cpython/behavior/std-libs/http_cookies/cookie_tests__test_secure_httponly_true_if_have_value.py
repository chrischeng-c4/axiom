# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "cookie_tests__test_secure_httponly_true_if_have_value"
# subject = "cpython.test_http_cookies.CookieTests.test_secure_httponly_true_if_have_value"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::CookieTests::test_secure_httponly_true_if_have_value
"""Auto-ported test: CookieTests::test_secure_httponly_true_if_have_value (CPython 3.12 oracle)."""


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
C.load('eggs=scrambled; httponly=foo; secure=bar; Path=/bacon')

assert C['eggs']['httponly']

assert C['eggs']['secure']

assert C['eggs']['httponly'] == 'foo'

assert C['eggs']['secure'] == 'bar'
print("CookieTests::test_secure_httponly_true_if_have_value: ok")
