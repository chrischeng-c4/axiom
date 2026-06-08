# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "cookie_tests__test_samesite_attrs"
# subject = "cpython.test_http_cookies.CookieTests.test_samesite_attrs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::CookieTests::test_samesite_attrs
"""Auto-ported test: CookieTests::test_samesite_attrs (CPython 3.12 oracle)."""


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
samesite_values = ['Strict', 'Lax', 'strict', 'lax']
for val in samesite_values:
    C = cookies.SimpleCookie('Customer="WILE_E_COYOTE"')
    C['Customer']['samesite'] = val

    assert C.output() == 'Set-Cookie: Customer="WILE_E_COYOTE"; SameSite=%s' % val
    C = cookies.SimpleCookie()
    C.load('Customer="WILL_E_COYOTE"; SameSite=%s' % val)

    assert C['Customer']['samesite'] == val
print("CookieTests::test_samesite_attrs: ok")
