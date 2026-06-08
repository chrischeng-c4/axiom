# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "cookie_tests__test_unquote"
# subject = "cpython.test_http_cookies.CookieTests.test_unquote"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::CookieTests::test_unquote
"""Auto-ported test: CookieTests::test_unquote (CPython 3.12 oracle)."""


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
cases = [('a="b=\\""', 'b="'), ('a="b=\\\\"', 'b=\\'), ('a="b=\\="', 'b=='), ('a="b=\\n"', 'b=n'), ('a="b=\\042"', 'b="'), ('a="b=\\134"', 'b=\\'), ('a="b=\\377"', 'b=ÿ'), ('a="b=\\400"', 'b=400'), ('a="b=\\42"', 'b=42'), ('a="b=\\\\042"', 'b=\\042'), ('a="b=\\\\134"', 'b=\\134'), ('a="b=\\\\\\""', 'b=\\"'), ('a="b=\\\\\\042"', 'b=\\"'), ('a="b=\\134\\""', 'b=\\"'), ('a="b=\\134\\042"', 'b=\\"')]
for encoded, decoded in cases:
    C = cookies.SimpleCookie()
    C.load(encoded)

    assert C['a'].value == decoded
print("CookieTests::test_unquote: ok")
