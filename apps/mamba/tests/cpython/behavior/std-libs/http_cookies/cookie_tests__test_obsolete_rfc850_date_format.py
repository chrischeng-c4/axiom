# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "cookie_tests__test_obsolete_rfc850_date_format"
# subject = "cpython.test_http_cookies.CookieTests.test_obsolete_rfc850_date_format"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::CookieTests::test_obsolete_rfc850_date_format
"""Auto-ported test: CookieTests::test_obsolete_rfc850_date_format (CPython 3.12 oracle)."""


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
test_cases = [{'data': 'key=value; expires=Saturday, 01-Jan-83 00:00:00 GMT', 'output': 'Saturday, 01-Jan-83 00:00:00 GMT'}, {'data': 'key=value; expires=Friday, 19-Nov-82 16:59:30 GMT', 'output': 'Friday, 19-Nov-82 16:59:30 GMT'}, {'data': 'key=value; expires=Sunday, 06-Nov-94 08:49:37 GMT', 'output': 'Sunday, 06-Nov-94 08:49:37 GMT'}, {'data': 'key=value; expires=Wednesday, 09-Nov-94 08:49:37 GMT', 'output': 'Wednesday, 09-Nov-94 08:49:37 GMT'}, {'data': 'key=value; expires=Friday, 11-Nov-94 08:49:37 GMT', 'output': 'Friday, 11-Nov-94 08:49:37 GMT'}, {'data': 'key=value; expires=Monday, 14-Nov-94 08:49:37 GMT', 'output': 'Monday, 14-Nov-94 08:49:37 GMT'}]
for case in test_cases:
    C = cookies.SimpleCookie()
    C.load(case['data'])
    cookie_name = case['data'].split('=')[0]

    assert cookie_name in C

    assert C[cookie_name].get('expires') == case['output']
print("CookieTests::test_obsolete_rfc850_date_format: ok")
