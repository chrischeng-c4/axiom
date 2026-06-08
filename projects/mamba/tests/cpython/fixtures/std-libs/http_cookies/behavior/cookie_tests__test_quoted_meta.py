# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "cookie_tests__test_quoted_meta"
# subject = "cpython.test_http_cookies.CookieTests.test_quoted_meta"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::CookieTests::test_quoted_meta
"""Auto-ported test: CookieTests::test_quoted_meta (CPython 3.12 oracle)."""


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
C.load('Customer="WILE_E_COYOTE"; Version="1"; Path="/acme"')

assert C['Customer'].value == 'WILE_E_COYOTE'

assert C['Customer']['version'] == '1'

assert C['Customer']['path'] == '/acme'

assert C.output(['path']) == 'Set-Cookie: Customer="WILE_E_COYOTE"; Path=/acme'

assert C.js_output() == '\n        <script type="text/javascript">\n        <!-- begin hiding\n        document.cookie = "Customer=\\"WILE_E_COYOTE\\"; Path=/acme; Version=1";\n        // end hiding -->\n        </script>\n        '

assert C.js_output(['path']) == '\n        <script type="text/javascript">\n        <!-- begin hiding\n        document.cookie = "Customer=\\"WILE_E_COYOTE\\"; Path=/acme";\n        // end hiding -->\n        </script>\n        '
print("CookieTests::test_quoted_meta: ok")
