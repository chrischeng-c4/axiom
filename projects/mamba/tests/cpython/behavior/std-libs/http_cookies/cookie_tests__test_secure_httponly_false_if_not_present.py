# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "cookie_tests__test_secure_httponly_false_if_not_present"
# subject = "cpython.test_http_cookies.CookieTests.test_secure_httponly_false_if_not_present"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_http_cookies.py::CookieTests::test_secure_httponly_false_if_not_present
"""Auto-ported test: CookieTests::test_secure_httponly_false_if_not_present (CPython 3.12 oracle)."""


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
C.load('eggs=scrambled; Path=/bacon')

assert not C['eggs']['httponly']

assert not C['eggs']['secure']
print("CookieTests::test_secure_httponly_false_if_not_present: ok")
