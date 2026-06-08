# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "morsel_tests__test_setter"
# subject = "cpython.test_http_cookies.MorselTests.test_setter"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::MorselTests::test_setter
"""Auto-ported test: MorselTests::test_setter (CPython 3.12 oracle)."""


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
M = cookies.Morsel()
for i in M._reserved:

    try:
        M.set(i, '%s_value' % i, '%s_value' % i)
        raise AssertionError('expected cookies.CookieError')
    except cookies.CookieError:
        pass
for i in 'thou cast _the- !holy! ^hand| +*grenade~'.split():
    M['path'] = '/foo'
    M.set(i, '%s_val' % i, '%s_coded_val' % i)

    assert M.key == i

    assert M.value == '%s_val' % i

    assert M.coded_value == '%s_coded_val' % i

    assert M.output() == 'Set-Cookie: %s=%s; Path=/foo' % (i, '%s_coded_val' % i)
    expected_js_output = '\n        <script type="text/javascript">\n        <!-- begin hiding\n        document.cookie = "%s=%s; Path=/foo";\n        // end hiding -->\n        </script>\n        ' % (i, '%s_coded_val' % i)

    assert M.js_output() == expected_js_output
for i in ['foo bar', 'foo@bar']:

    try:
        M.set(i, '%s_value' % i, '%s_value' % i)
        raise AssertionError('expected cookies.CookieError')
    except cookies.CookieError:
        pass
print("MorselTests::test_setter: ok")
