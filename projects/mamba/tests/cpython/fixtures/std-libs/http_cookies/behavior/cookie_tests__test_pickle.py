# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "cookie_tests__test_pickle"
# subject = "cpython.test_http_cookies.CookieTests.test_pickle"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::CookieTests::test_pickle
"""Auto-ported test: CookieTests::test_pickle (CPython 3.12 oracle)."""


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
rawdata = 'Customer="WILE_E_COYOTE"; Path=/acme; Version=1'
expected_output = 'Set-Cookie: %s' % rawdata
C = cookies.SimpleCookie()
C.load(rawdata)

assert C.output() == expected_output
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    C1 = pickle.loads(pickle.dumps(C, protocol=proto))

    assert C1.output() == expected_output
print("CookieTests::test_pickle: ok")
