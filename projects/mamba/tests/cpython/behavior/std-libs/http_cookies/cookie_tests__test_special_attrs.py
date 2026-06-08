# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "cookie_tests__test_special_attrs"
# subject = "cpython.test_http_cookies.CookieTests.test_special_attrs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::CookieTests::test_special_attrs
"""Auto-ported test: CookieTests::test_special_attrs (CPython 3.12 oracle)."""


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
C = cookies.SimpleCookie('Customer="WILE_E_COYOTE"')
C['Customer']['expires'] = 0

assert C.output().endswith('GMT')
C = cookies.SimpleCookie()
C.load('Customer="W"; expires=Wed, 01 Jan 2010 00:00:00 GMT')

assert C['Customer']['expires'] == 'Wed, 01 Jan 2010 00:00:00 GMT'
C = cookies.SimpleCookie()
C.load('Customer="W"; expires=Wed, 01 Jan 98 00:00:00 GMT')

assert C['Customer']['expires'] == 'Wed, 01 Jan 98 00:00:00 GMT'
C = cookies.SimpleCookie('Customer="WILE_E_COYOTE"')
C['Customer']['max-age'] = 10

assert C.output() == 'Set-Cookie: Customer="WILE_E_COYOTE"; Max-Age=10'
print("CookieTests::test_special_attrs: ok")
