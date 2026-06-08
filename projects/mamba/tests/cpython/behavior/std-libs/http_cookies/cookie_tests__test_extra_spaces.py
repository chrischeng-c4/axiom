# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "cookie_tests__test_extra_spaces"
# subject = "cpython.test_http_cookies.CookieTests.test_extra_spaces"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::CookieTests::test_extra_spaces
"""Auto-ported test: CookieTests::test_extra_spaces (CPython 3.12 oracle)."""


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
C.load('eggs  =  scrambled  ;  secure  ;  path  =  bar   ; foo=foo   ')

assert C.output() == 'Set-Cookie: eggs=scrambled; Path=bar; Secure\r\nSet-Cookie: foo=foo'
print("CookieTests::test_extra_spaces: ok")
