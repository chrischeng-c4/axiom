# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "cookie_tests__test_comment_quoting"
# subject = "cpython.test_http_cookies.CookieTests.test_comment_quoting"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::CookieTests::test_comment_quoting
"""Auto-ported test: CookieTests::test_comment_quoting (CPython 3.12 oracle)."""


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
c = cookies.SimpleCookie()
c['foo'] = '©'

assert str(c['foo']) == 'Set-Cookie: foo="\\251"'
c['foo']['comment'] = 'comment ©'

assert str(c['foo']) == 'Set-Cookie: foo="\\251"; Comment="comment \\251"'
print("CookieTests::test_comment_quoting: ok")
