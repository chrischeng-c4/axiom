# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "cookie_tests__test_basic"
# subject = "cpython.test_http_cookies.CookieTests.test_basic"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::CookieTests::test_basic
"""Auto-ported test: CookieTests::test_basic (CPython 3.12 oracle)."""


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
cases = [{'data': 'chips=ahoy; vienna=finger', 'dict': {'chips': 'ahoy', 'vienna': 'finger'}, 'repr': "<SimpleCookie: chips='ahoy' vienna='finger'>", 'output': 'Set-Cookie: chips=ahoy\nSet-Cookie: vienna=finger'}, {'data': 'keebler="E=mc2; L=\\"Loves\\"; fudge=\\012;"', 'dict': {'keebler': 'E=mc2; L="Loves"; fudge=\n;'}, 'repr': '<SimpleCookie: keebler=\'E=mc2; L="Loves"; fudge=\\n;\'>', 'output': 'Set-Cookie: keebler="E=mc2; L=\\"Loves\\"; fudge=\\012;"'}, {'data': 'keebler=E=mc2', 'dict': {'keebler': 'E=mc2'}, 'repr': "<SimpleCookie: keebler='E=mc2'>", 'output': 'Set-Cookie: keebler=E=mc2'}, {'data': 'key:term=value:term', 'dict': {'key:term': 'value:term'}, 'repr': "<SimpleCookie: key:term='value:term'>", 'output': 'Set-Cookie: key:term=value:term'}, {'data': 'a=b; c=[; d=r; f=h', 'dict': {'a': 'b', 'c': '[', 'd': 'r', 'f': 'h'}, 'repr': "<SimpleCookie: a='b' c='[' d='r' f='h'>", 'output': '\n'.join(('Set-Cookie: a=b', 'Set-Cookie: c=[', 'Set-Cookie: d=r', 'Set-Cookie: f=h'))}]
for case in cases:
    C = cookies.SimpleCookie()
    C.load(case['data'])

    assert repr(C) == case['repr']

    assert C.output(sep='\n') == case['output']
    for k, v in sorted(case['dict'].items()):

        assert C[k].value == v
print("CookieTests::test_basic: ok")
